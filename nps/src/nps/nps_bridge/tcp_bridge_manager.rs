use super::tcp_bridge::{TCPBridge, TCPBridgeInfo};
use crate::dao::channel_dao::Channel;
use crate::model::data_io_len::AtomicDataIOLen;
use crate::nps::nps_pool::tcp_pool_manager;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Notify;
//TCP桥接会话管理

// // 当前正在通信的桥接
// var bridgeMap = make(map[*TCPBridge]bool)
// var bridgeLock sync.Mutex

// // 当前桥接数量
// fn GetBridgeCount() int {
// 	count := 0
// 	bridgeLock.Lock()
// 	count = len(bridgeMap)
// 	bridgeLock.Unlock()
// 	return count
// }

// // 获取当前桥接列表
// fn GetBridgeList() []TCPBridge {
// 	list := []TCPBridge{}
// 	bridgeLock.Lock()
// 	for item := range bridgeMap {
// 		list = append(list, *item)
// 	}
// 	bridgeLock.Unlock()
// 	return list
// }

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
) {
    //NPS客户端Socket
    let Some(npc_pool_tcp) = tcp_pool_manager::get_and_add_pool(channel.client_id).await else {
        // println!("-->客户端: {}没有可用的连接池。", channel.client_id);

        //这里无需关闭，生命周期结束之后会自动关闭
        // tcp.shutdown().await?;
        return;
    };
    let bridge = TCPBridge {
        bridge_info_map,
        target_port: channel.target_port.clone(),
        security_state: channel.security_state,
        proxy_tcp,
        client_tcp: npc_pool_tcp,
        channel_data_len: data_len,
        channel_closer: close_notify,
    };
    tokio::spawn(async {
        if let Err(e) = bridge.start().await {
            println!("桥接通信接发生了错误:{:?}", e);
        }
    });
}
