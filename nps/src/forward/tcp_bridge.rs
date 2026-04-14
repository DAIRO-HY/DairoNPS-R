use crate::model::data_io_len::AtomicDataIOLen;
use crate::nps_error::NpsError;
use dashmap::DashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
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

pub struct TcpBridgeParam {
    pub is_stats_traffic: bool,
    pub target_port: String,
    pub bridge_map: Arc<DashMap<u64, TCPBridgeInfo>>,
    pub proxy_tcp: TcpStream,
    pub data_len: AtomicDataIOLen,
    pub closer: Arc<Notify>,
    pub bridge_count: Arc<AtomicUsize>
}

//用来生成当前桥接唯一标识
static NEXT_KEY: AtomicU64 = AtomicU64::new(1);

/**
 * 开始会话
 * @param client 客户端DTO
 * @param channel 隧道信息
 * @param proxySocket 代理服务端Socket
 * @param clientSocket 内网穿透客户端Socket
 */
pub async fn ready(param: TcpBridgeParam) {
    tokio::spawn(async move {
        if let Err(e) = start(param).await {
            println!("桥接通信接发生了错误:{:?}", e);
        }
    });
}

/**
 * 开始桥接传输数据
 */
async fn start(mut param: TcpBridgeParam) -> Result<(), NpsError> {

    //桥接数+1
    param.bridge_count.fetch_add(1, Ordering::Relaxed);

    // 建立连接
    let target_tcp = match TcpStream::connect(param.target_port).await {
        Ok(v) => v,
        Err(e) => {
            //与目标服务器连接失败时，直接关闭
            let _ = param.proxy_tcp.shutdown().await;

            //桥接数-1
            param.bridge_count.fetch_sub(1, Ordering::Relaxed);
            return Err(NpsError::IoError(e));
        }
    };
    if !param.is_stats_traffic {
        // 不需要实时统计流量
        let result = copy(
            target_tcp,
            param.proxy_tcp,
            param.closer,
            param.data_len,
        )
        .await;

        //桥接数-1
        param.bridge_count.fetch_sub(1, Ordering::Relaxed);
        return result;
    }

    //统计当前桥接流量
    let bridge_data_len = AtomicDataIOLen::new();
    let key = NEXT_KEY.fetch_add(1, Ordering::Relaxed);

    //保存当前桥接信息，供监控使用
    param.bridge_map.insert(
        key,
        TCPBridgeInfo {
            data_len: param.data_len.clone(),
            create_time: 0,
            last_rw_time: 0,
        },
    );

    let (proxy_reader, proxy_writer) = io::split(param.proxy_tcp);
    let (target_reader, target_writer) = io::split(target_tcp);

    let p2t = proxy_to_target(
        bridge_data_len.clone(),
        param.data_len.clone(),
        proxy_reader,
        target_writer,
        param.closer.clone(),
    );
    let c2p = target_to_proxy(
        bridge_data_len,
        param.data_len,
        target_reader,
        proxy_writer,
        param.closer,
    );
    let result = try_join!(p2t, c2p);

    //桥接结束后,移除桥接信息
    param.bridge_map.remove(&key);

    //桥接数-1
    param.bridge_count.fetch_sub(1, Ordering::Relaxed);
    result?;
    Ok(())
}

/// 不需要实时统计流量，高性能模式
async fn copy(
    mut target_tcp: TcpStream,
    mut proxy_tcp: TcpStream,
    closer: Arc<Notify>,
    data_len: AtomicDataIOLen,
) -> Result<(), NpsError> {
    select! {
        _ = closer.notified() => {
            proxy_tcp.shutdown().await?;
            target_tcp.shutdown().await?;
            return Ok(());
        }
        rs = io::copy_bidirectional(&mut proxy_tcp, &mut target_tcp) =>{
            return match rs{
                Ok((in_len,out_len))=>{
                    data_len.add_in(in_len);
                    data_len.add_out(out_len);
                    Ok(())
                },
                Err(e) =>{
                    Err(NpsError::IoError(e))
                }
            }
        }
    }
}

async fn proxy_to_target(
    bridge_data_len: AtomicDataIOLen,
    forward_data_len: AtomicDataIOLen,
    mut proxy_reader: ReadHalf<TcpStream>,
    mut target_writer: WriteHalf<TcpStream>,
    closer: Arc<Notify>,
) -> io::Result<()> {
    let mut buf = [0u8; 4096];
    loop {
        select! {
            _ = closer.notified() => {
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
    closer: Arc<Notify>,
) -> io::Result<()> {
    let mut buf = [0u8; 4096];
    loop {
        select! {
            _ = closer.notified() => {
                break;
            }
            read_data = target_reader.read(&mut buf) =>{
                let n = read_data?;
                if n == 0 {
                    break;
                }
                bridge_data_len.add_out(n);
                forward_data_len.add_out(n);
                proxy_writer.write_all(&buf[..n]).await?;
            }
        }
    }
    //这里必须关闭客户端的输出流，否则对方无法感知到已经关闭连接了（写失败或者读失败没有必要调用shutdown()，即使调用大概率也是失败的，所以没有意义）
    proxy_writer.shutdown().await?;
    // println!("-->代理连接已关闭");
    Ok(())
}
