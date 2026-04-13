use crate::model::data_io_len::AtomicDataIOLen;
use crate::nps_error::NpsError;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Notify;
use tokio::{io, select, try_join};

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
    pub proxy_tcp: TcpStream,
    pub target_tcp: TcpStream,
    pub data_len: AtomicDataIOLen,
    pub closer: Arc<Notify>,
}

//用来生成当前桥接唯一标识
static NEXT_KEY: AtomicU64 = AtomicU64::new(1);

impl TCPBridge {
    /**
     * 开始桥接传输数据
     */
    pub async fn start(self) -> Result<(), NpsError> {
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
        let (target_reader, target_writer) = tokio::io::split(self.target_tcp);

        let p2t = Self::proxy_to_target(
            data_len.clone(),
            self.data_len.clone(),
            proxy_reader,
            target_writer,
            self.closer,
        );
        let c2p = Self::target_to_proxy(data_len, self.data_len, target_reader, proxy_writer);
        let result = try_join!(p2t, c2p);

        //桥接结束后,移除桥接信息
        self.bridge_info_map.remove(&key);
        result?;
        Ok(())
    }

    async fn proxy_to_target(
        bridge_data_len: AtomicDataIOLen,
        forward_data_len: AtomicDataIOLen,
        mut proxy_reader: ReadHalf<TcpStream>,
        mut target_writer: WriteHalf<TcpStream>,
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
                    bridge_data_len.add_in(n);
                    forward_data_len.add_in(n);
                    target_writer.write_all(&buf[..n]).await?;
                }
            }
        }

        //这里必须关闭客户端的输出流，否则对方无法感知到已经关闭连接了（写失败或者读失败没有必要调用shutdown()，即使调用大概率也是失败的，所以没有意义）
        target_writer.shutdown().await?;
        Ok(())
    }

    async fn target_to_proxy(
        bridge_data_len: AtomicDataIOLen,
        forward_data_len: AtomicDataIOLen,
        mut target_reader: ReadHalf<TcpStream>,
        mut proxy_writer: WriteHalf<TcpStream>,
    ) -> io::Result<()> {
        let mut buf = [0u8; 4096];
        loop {
            let n = target_reader.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            bridge_data_len.add_out(n);
            forward_data_len.add_out(n);
            proxy_writer.write_all(&buf[..n]).await?;
        }
        //这里必须关闭客户端的输出流，否则对方无法感知到已经关闭连接了（写失败或者读失败没有必要调用shutdown()，即使调用大概率也是失败的，所以没有意义）
        proxy_writer.shutdown().await?;
        // println!("-->代理连接已关闭");
        Ok(())
    }
}
