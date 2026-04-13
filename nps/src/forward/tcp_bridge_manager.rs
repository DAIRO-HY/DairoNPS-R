use super::tcp_bridge::TCPBridgeInfo;
use crate::dao::forward_dao::Forward;
use crate::forward::tcp_bridge;
use crate::model::data_io_len::AtomicDataIOLen;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Notify;

/**
 * 开始会话
 * @param client 客户端DTO
 * @param channel 隧道信息
 * @param proxySocket 代理服务端Socket
 * @param clientSocket 内网穿透客户端Socket
 */
pub async fn make_bridge(
    bridge_info_map: Arc<DashMap<u64, TCPBridgeInfo>>,
    forward: &Forward,
    mut proxy_tcp: TcpStream,
    data_len: AtomicDataIOLen,
    close_notify: Arc<Notify>,
    bridge_count: Arc<AtomicUsize>,
) {
    let target_port = forward.target_port.clone();
    let is_stats_traffic = forward.is_stats_traffic.clone();
    tokio::spawn(async move {
        // 建立连接
        let target_tcp = match TcpStream::connect(target_port).await {
            Ok(v) => v,
            Err(e) => {
                //与目标服务器连接失败时，直接关闭
                let _ = proxy_tcp.shutdown().await;
                return;
            }
        };

        //并发数+1
        bridge_count.fetch_add(1, Ordering::Relaxed);
        if let Err(e) = tcp_bridge::start(
            is_stats_traffic,
            bridge_info_map,
            proxy_tcp,
            target_tcp,
            data_len,
            close_notify,
        )
        .await
        {
            println!("桥接通信接发生了错误:{:?}", e);
        }

        //并发数-1
        bridge_count.fetch_sub(1, Ordering::Relaxed);
    });
}
