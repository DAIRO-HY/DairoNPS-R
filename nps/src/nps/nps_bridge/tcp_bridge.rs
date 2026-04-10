use crate::model::data_io_len::AtomicDataIOLen;
use crate::util::security_util::SERVER_SECURITY_KEY;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::vec::Vec;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Notify;
use tokio::{io, select, try_join};
use crate::nps::nps_error::NpsError;

// TCPBridge TCP桥接信息
#[derive(Debug)]
pub struct TCPBridgeInfo {
    pub data_len: AtomicDataIOLen,

    // 创建时间(毫秒)
    pub create_time: u64,

    // 记录最后通信时间(毫秒)
    pub last_rw_time: u64,
}

// TCPBridge TCP桥接
#[derive(Debug)]
pub struct TCPBridge {
    pub bridge_info_map: Arc<DashMap<u64, TCPBridgeInfo>>,

    //目标端口(ip:端口)
    pub target_port: String,

    //是否加密传输
    pub security_state: i64,
    pub proxy_tcp: TcpStream,
    pub client_tcp: TcpStream,
    pub channel_data_len: AtomicDataIOLen,
    pub channel_closer: Arc<Notify>,
}

//用来生成当前桥接唯一标识
static NEXT_KEY: AtomicU64 = AtomicU64::new(1);

impl TCPBridge {
    /**
     * 开始桥接传输数据
     */
    pub async fn start(self) -> Result<(),NpsError> {
        //统计当前桥接流量
        let data_len = AtomicDataIOLen::new();
        let key = NEXT_KEY.fetch_add(1, Ordering::Relaxed);

        //保存当前桥接信息，供监控使用
        self.bridge_info_map.insert(
            key,
            TCPBridgeInfo {
                data_len: data_len.clone(),
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
            self.security_state == 1,
            data_len.clone(),
            self.channel_data_len.clone(),
            proxy_reader,
            client_writer,
            self.channel_closer,
        );
        let c2p = Self::client_to_proxy(
            self.security_state == 1,
            data_len,
            self.channel_data_len,
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
        need_encryption: bool,
        data_len: AtomicDataIOLen,
        channel_data_len: AtomicDataIOLen,
        mut proxy_reader: ReadHalf<TcpStream>,
        mut client_writer: WriteHalf<TcpStream>,
        close_notify: Arc<Notify>,
    ) -> io::Result<()> {
        let mut buf = [0u8; 4096];
        loop {
            select! {
                _ = close_notify.notified() => {
                    // println!("-->接收到关闭通知：退出 proxy_to_client 循环");
                    break;
                }
                read_data = proxy_reader.read(&mut buf) =>{
                    let n = read_data?;
                    if n == 0 {
                        break;
                    }
                    data_len.add_in(n);
                    channel_data_len.add_in(n);
                    if need_encryption{//需要加密处理
                        for b in &mut buf[..n] {
                            *b = SERVER_SECURITY_KEY[*b as usize];
                        }
                    }
                    client_writer.write_all(&buf[..n]).await?;
                }
            }
        }

        //这里必须关闭客户端的输出流，否则对方无法感知到已经关闭连接了（写失败或者读失败没有必要调用shutdown()，即使调用大概率也是失败的，所以没有意义）
        client_writer.shutdown().await?;
        // println!("-->客户端连接已关闭");
        Ok(())
    }

    async fn client_to_proxy(
        need_encryption: bool,
        data_len: AtomicDataIOLen,
        channel_data_len: AtomicDataIOLen,
        mut client_reader: ReadHalf<TcpStream>,
        mut proxy_writer: WriteHalf<TcpStream>,
    ) -> io::Result<()> {
        let mut buf = [0u8; 4096];
        loop {
            let n = client_reader.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            data_len.add_out(n);
            channel_data_len.add_out(n);
            if need_encryption {
                //需要解密处理
                for b in &mut buf[..n] {
                    *b = SERVER_SECURITY_KEY[*b as usize];
                }
            }
            proxy_writer.write_all(&buf[..n]).await?;
        }
        //这里必须关闭客户端的输出流，否则对方无法感知到已经关闭连接了（写失败或者读失败没有必要调用shutdown()，即使调用大概率也是失败的，所以没有意义）
        proxy_writer.shutdown().await?;
        // println!("-->代理连接已关闭");
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
}
