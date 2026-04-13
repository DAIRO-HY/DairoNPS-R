use super::tcp_bridge::{TCPBridge, TCPBridgeInfo};
use crate::model::data_io_len::AtomicDataIOLen;
use crate::nps::nps_pool::tcp_pool_manager;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Notify;
use crate::dao::forward_dao::Forward;

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
) {
    let target_port = forward.target_port.clone();
    tokio::spawn(async {

        // 1. 建立连接
        let mut target_tcp = match TcpStream::connect(target_port).await{
            Ok(v) => v,
            Err(e)=>{//与目标服务器连接失败时，直接关闭
                let _ = proxy_tcp.shutdown().await;
                return
            },
        };
        let bridge = TCPBridge {
            bridge_info_map,
            proxy_tcp,
            target_tcp,
            data_len,
            closer: close_notify,
        };
        if let Err(e) = bridge.start().await {
            println!("桥接通信接发生了错误:{:?}", e);
        }
    });
}
