use crate::model::data_total::DataTotal;
use dashmap::DashMap;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use std::vec::Vec;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::{io, try_join};

// TCPBridge TCP桥接信息
pub struct TCPBridgeInfo {
    pub data_total: DataTotal,

    // 创建时间(毫秒)
    pub create_time: u64,

    // 记录最后通信时间(毫秒)
    pub last_rw_time: u64,
}

// TCPBridge TCP桥接
pub struct TCPBridge {
    pub bridge_info_map: Arc<DashMap<u64, TCPBridgeInfo>>,

    //目标端口(ip:端口)
    pub target_port: String,

    //是否加密传输
    pub security_state: i8,
    pub proxy_tcp: TcpStream,
    pub client_tcp: TcpStream,
    pub channel_data_total: DataTotal,
}

//用来生成当前桥接唯一标识
static NEXT_KEY: AtomicU64 = AtomicU64::new(1);

static PATH_SET: OnceLock<tokio::sync::Mutex<HashSet<String>>> = OnceLock::new();

impl TCPBridge {
    /**
     * 开始桥接传输数据
     */
    pub async fn start(self) -> io::Result<()> {
        //统计当前桥接流量
        let data_total = DataTotal::new();
        let key = NEXT_KEY.fetch_add(1, Ordering::Relaxed);

        //保存当前桥接信息，供监控使用
        self.bridge_info_map.insert(
            key,
            TCPBridgeInfo {
                data_total: data_total.clone(),
                create_time: 0,
                last_rw_time: 0,
            },
        );

        let (proxy_reader, proxy_writer) = tokio::io::split(self.proxy_tcp);
        let (client_reader, mut client_writer) = tokio::io::split(self.client_tcp);

        //将加密类型及目标端口 格式:加密状态|端口  1|80   1|127.0.0.1:80
        //1:加密  0:不加密
        let mut header = String::new();
        header.push_str(&(self.security_state.to_string()));
        header.push('|');
        header.push_str(&(self.target_port.to_string()));

        //发送目标端口信息
        Self::send_header_to_client(header, &mut client_writer).await?;

        let p2c = Self::proxy_to_client(
            data_total.clone(),
            self.channel_data_total.clone(),
            proxy_reader,
            client_writer,
        );
        let c2p = Self::client_to_proxy(
            data_total,
            self.channel_data_total,
            client_reader,
            proxy_writer,
        );
        let result = try_join!(p2c, c2p);

        //桥接结束后,移除桥接信息
        self.bridge_info_map.remove(&key);
        // println!("-->桥接数据传输结束");
        result?;
        Ok(())
    }

    async fn proxy_to_client(
        data_total: DataTotal,
        channel_data_total: DataTotal,
        mut proxy_reader: ReadHalf<TcpStream>,
        mut client_writer: WriteHalf<TcpStream>,
    ) -> io::Result<()> {
        let mut content_buf = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            let n = proxy_reader.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            data_total.add_in(n);
            channel_data_total.add_in(n);
            content_buf.extend_from_slice(&buf[..n]);
            client_writer.write_all(&buf[..n]).await?;
        }

        let content = String::from_utf8_lossy(&content_buf);
        println!("--------------------------------------------------------->");
        let head_path_list: Vec<String> = content
            .split("\n")
            .filter_map(|it| {
                if it.starts_with("GET /")
                    || it.starts_with("POST /")
                    || it.starts_with("PUT /")
                    || it.starts_with("DELETE /")
                {
                    let host_info_arr: Vec<&str> = it.split(" ").collect();
                    if host_info_arr.len() == 3 {
                        let mut path = host_info_arr[1].to_string();
                        let query_flag_index = path.find("?");
                        if let Some(index) = query_flag_index {
                            path = path[..index].to_string();
                        }
                        if path.ends_with(".js")
                            || path.ends_with(".css")
                            || path.ends_with(".ico")
                            || path.ends_with(".png")
                            || path.ends_with(".jpg")
                            || path.ends_with(".jpeg")
                            || path.ends_with(".woff2")
                            || path.ends_with(".woff")
                            || path.ends_with(".html")
                            || path.ends_with(".json")
                        {
                            return None;
                        }
                        return Some(path);
                    }
                }
                None
            })
            .collect();

        let path_set = PATH_SET.get_or_init(|| Mutex::new(HashSet::new()));
        let mut path_set = path_set.lock().await;
        path_set.extend(head_path_list);
        path_set.iter().for_each(|it| println!("{}", it));
        drop(path_set);
        println!("--------------------------------------------------------->");

        //这里必须关闭客户端的输出流，否则对方无法感知到已经关闭连接了（写失败或者读失败没有必要调用shutdown()，即使调用大概率也是失败的，所以没有意义）
        client_writer.shutdown().await?;
        println!("-->客户端连接已关闭");
        Ok(())
    }

    async fn client_to_proxy(
        data_total: DataTotal,
        channel_data_total: DataTotal,
        mut client_reader: ReadHalf<TcpStream>,
        mut proxy_writer: WriteHalf<TcpStream>,
    ) -> io::Result<()> {
        let mut buf = [0u8; 4096];
        loop {
            let n = client_reader.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            data_total.add_out(n);
            channel_data_total.add_out(n);
            proxy_writer.write_all(&buf[..n]).await?;
        }
        //这里必须关闭客户端的输出流，否则对方无法感知到已经关闭连接了（写失败或者读失败没有必要调用shutdown()，即使调用大概率也是失败的，所以没有意义）
        proxy_writer.shutdown().await?;
        println!("-->代理连接已关闭");
        Ok(())
    }

    /**
     * 发送目标端口信息
     */
    async fn send_header_to_client(
        header: String,
        client_writer: &mut WriteHalf<TcpStream>,
    ) -> io::Result<()> {
        let mut data = Vec::with_capacity(header.len() + 1);
        data.push(header.len() as u8); //写入数据长度标识
        data.extend_from_slice(header.as_bytes()); //写入数据

        client_writer.write_all(&data).await?;
        Ok(())
    }

    /**
     * 从代理服务接收数据发送到客户端
     */
    async fn receiveByProxySendToClient(&mut self) {
        // let mut buf = [0u8; 1024];
        // loop {
        //     match self.proxy_reader.read(&mut buf).await {
        //         Ok(0) => {
        //             println!("s:连接已关闭");
        //             return;
        //         }
        //         Ok(n) => {
        //             // tx.send(&buf[..n]).await.unwrap();

        //             // let received = String::from_utf8_lossy(&buf[..n]);
        //             // println!("s:收到客户端消息: {}", received);
        //             // tx.send("-->server:我已经收到你的消息".to_string()).await.unwrap();

        //             self.client_writer.write_all(&buf[..n]).await.unwrap();
        //         }
        //         Err(e) => {
        //             eprintln!("s:读取错误: {}", e);
        //             return;
        //         }
        //     }
        // }

        // data := make([]uint8, NPSConstant.READ_CACHE_SIZE)
        // for {
        //     n, readErr := mine.ProxyTCP.Read(data)
        //     if n > 0 {
        //
        //         //记录最后通信时间
        //         mine.LastRWTime = time.Now().UnixMilli()
        //
        //         //原子递增
        //         atomic.AddInt64(&mine.channelDataSize.InData, int64(n))
        //         if mine.Channel.SecurityState == 1 { //加密数据
        //             SecurityUtil.Mapping(data, n)
        //         }
        //
        //         //将读取到的数据立即发送客户端
        //         writeErr := WriterUtil.WriteFull(mine.ClientTCP, data[:n])
        //         if writeErr != nil {
        //             break
        //         }
        //     }
        //     if readErr != nil {
        //         break
        //     }
        // }
        //
        // //关闭客户端的输出流
        // mine.ClientTCP.(*net.TCPConn).CloseWrite()
        //
        // //关闭代理端的输入流
        // mine.ProxyTCP.(*net.TCPConn).CloseRead()
        //
        // //标记代理连接读操作已经关闭
        // mine.proxyInIsClosed = true
        // mine.recycle()
    }
    //
    // // 从客户端接收发送到代理服务器
    // func (mine *TCPBridge) receiveByClientSendToProxy() {
    //     data := make([]uint8, NPSConstant.READ_CACHE_SIZE)
    //     for {
    //         n, readErr := mine.ClientTCP.Read(data)
    //         if n > 0 {
    //
    //             //记录最后通信时间
    //             mine.LastRWTime = time.Now().UnixMilli()
    //
    //             //出网统计 原子递增
    //             atomic.AddInt64(&mine.channelDataSize.OutData, int64(n))
    //             if mine.Channel.SecurityState == 1 { //加密数据
    //                 SecurityUtil.Mapping(data, n)
    //             }
    //
    //             //将读取到的数据立即发送客户端
    //             writeErr := WriterUtil.WriteFull(mine.ProxyTCP, data[:n])
    //             if writeErr != nil {
    //                 break
    //             }
    //         }
    //         if readErr != nil {
    //             break
    //         }
    //     }
    //
    //     //关闭客户端的输出流
    //     mine.ProxyTCP.(*net.TCPConn).CloseWrite()
    //
    //     //关闭代理端的输入流
    //     mine.ClientTCP.(*net.TCPConn).CloseRead()
    //
    //     //标记客户端读操作已经关闭
    //     mine.clientInIsClosed = true
    //     mine.recycle()
    // }
    //
    // /**
    // * 资源回收
    //  */
    // func (mine *TCPBridge) recycle() {
    //     if mine.proxyInIsClosed && mine.clientInIsClosed {
    //         mine.ClientTCP.Close()
    //         mine.ProxyTCP.Close()
    //         removeBridge(mine)
    //     }
    // }
    //
    // /**
    // * 关闭连接
    //  */
    // func (mine *TCPBridge) shutdown() {
    //     mine.ClientTCP.Close()
    //     mine.ProxyTCP.Close()
    // }
}
