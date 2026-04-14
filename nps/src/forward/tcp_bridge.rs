use crate::{application, forward};
use crate::forward::TCPBridging;
use crate::model::data_io_len::AtomicDataIOLen;
use crate::nps_error::NpsError;
use crate::util::time_util;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Notify;
use tokio::{io, select, try_join};

pub struct TcpBridgeParam {
    pub ip: String, // 代理客户端ip地址
    pub forward_id: i64,
    pub is_stats_traffic: bool,
    pub target_port: String,
    pub proxy_tcp: TcpStream,
    pub data_len: AtomicDataIOLen,
    pub closer: Arc<Notify>,
    pub bridge_count: Arc<AtomicUsize>,
}

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
        let result = copy(target_tcp, param.proxy_tcp, param.closer, param.data_len).await;

        //桥接数-1
        param.bridge_count.fetch_sub(1, Ordering::Relaxed);
        return result;
    }

    //统计当前桥接流量
    let data_len = AtomicDataIOLen::new();
    let tag = application::BRIDGE_NEXT_TAG.fetch_add(1, Ordering::Relaxed);
    let bridge_closer = Arc::new(Notify::new());
    let last_rw_time = Arc::new(AtomicU64::new(time_util::current_millis()));

    //保存当前桥接信息，供监控使用
    forward::FORWARD_BRIDGING_MAP.insert(
        tag,
        TCPBridging {
            ip: param.ip,
            forward_id: param.forward_id,
            data_len: data_len.clone(),
            create_time: time_util::current_millis(),
            last_rw_time: last_rw_time.clone(),
            closer: bridge_closer.clone(),
        },
    );

    let (proxy_reader, proxy_writer) = io::split(param.proxy_tcp);
    let (target_reader, target_writer) = io::split(target_tcp);

    let p2t = proxy_to_target(
        data_len.clone(),
        param.data_len.clone(),
        proxy_reader,
        target_writer,
        param.closer.clone(),
        bridge_closer.clone(),
        last_rw_time.clone(),
    );
    let c2p = target_to_proxy(
        data_len,
        param.data_len,
        target_reader,
        proxy_writer,
        param.closer,
        bridge_closer,
        last_rw_time,
    );
    let result = try_join!(p2t, c2p);

    //桥接结束后,移除桥接信息
    forward::FORWARD_BRIDGING_MAP.remove(&tag);

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
    bridge_closer: Arc<Notify>,
    last_rw_time: Arc<AtomicU64>,
) -> io::Result<()> {
    let mut buf = [0u8; 1024 * 8];
    loop {
        select! {
            _ = closer.notified() => {
                // println!("-->接收到关闭通知：退出 proxy_to_client 循环");
                break;
            }
            _ = bridge_closer.notified() => {
                break;
            }
            read_data = proxy_reader.read(&mut buf) =>{
                let n = read_data?;
                if n == 0 {
                    break;
                }

                //记录最后一次读写时间
                last_rw_time.store(time_util::current_millis(),Ordering::Relaxed);
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
    bridge_closer: Arc<Notify>,
    last_rw_time: Arc<AtomicU64>,
) -> io::Result<()> {
    let mut buf = [0u8; 1024 * 8];
    loop {
        select! {
            _ = closer.notified() => {
                break;
            }
            _ = bridge_closer.notified() => {
                break;
            }
            read_data = target_reader.read(&mut buf) =>{
                let n = read_data?;
                if n == 0 {
                    break;
                }

                //记录最后一次读写时间
                last_rw_time.store(time_util::current_millis(),Ordering::Relaxed);
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
