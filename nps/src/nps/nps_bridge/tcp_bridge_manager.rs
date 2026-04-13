use super::tcp_bridge::{TCPBridgeInfo};
use crate::dao::channel_dao::Channel;
use crate::model::data_io_len::AtomicDataIOLen;
use crate::nps::nps_pool::tcp_pool_manager;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::net::TcpStream;
use tokio::sync::Notify;
use crate::nps::nps_bridge::tcp_bridge;

/**
 * 开始会话
 * @param client 客户端DTO
 * @param channel 隧道信息
 * @param proxySocket 代理服务端Socket
 * @param clientSocket 内网穿透客户端Socket
 */
pub async fn make_bridge(
    bridge_info_map: Arc<DashMap<u64, TCPBridgeInfo>>,
    channel: &Channel,
    proxy_tcp: TcpStream,
    data_len: AtomicDataIOLen,
    close_notify: Arc<Notify>,
    bridge_count:Arc<AtomicUsize>,
) {
    //NPS客户端Socket
    let Some(npc_pool_tcp) = tcp_pool_manager::get_and_add_pool(channel.client_id).await else {
        return;
    };
    let target_port = channel.target_port.clone();
    let is_stats_traffic = channel.is_stats_traffic.clone();
    let security_state = channel.security_state.clone();
    tokio::spawn(async move{

        //并发数+1
        bridge_count.fetch_add(1, Ordering::Relaxed);
        if let Err(e) = tcp_bridge::start(
            is_stats_traffic,
            bridge_info_map,
            target_port,
            security_state,
            proxy_tcp,
            npc_pool_tcp,
            data_len,
            close_notify,
        ).await {
            println!("桥接通信接发生了错误:{:?}", e);
        }

        //并发数-1
        bridge_count.fetch_sub(1,Ordering::Relaxed);
    });
}
