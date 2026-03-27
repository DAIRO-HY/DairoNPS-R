use std::sync::Arc;
use dashmap::DashMap;
use super::tcp_bridge::{TCPBridge, TCPBridgeInfo};
use crate::dao::channel_dao::Channel;
use crate::model::data_total::DataTotal;
use crate::nps::nps_pool::tcp_pool_manager;
use tokio::net::TcpStream;

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
pub async fn make_bridge(bridge_info_map:Arc<DashMap<u64,TCPBridgeInfo>>,channel: &Channel, proxy_tcp: TcpStream, data_total: DataTotal) {
    //NPS客户端Socket
    let npc_pool_tcp = tcp_pool_manager::get_and_add_pool(channel.client_id).await;
    if npc_pool_tcp.is_none() {
        println!("-->客户端: {}没有可用的连接池。", channel.client_id);

        //这里无需关闭，生命周期结束之后会自动关闭
        // tcp.shutdown().await?;
        return;
    }
    let npc_pool_tcp = npc_pool_tcp.unwrap();
    let bridge = TCPBridge {
        bridge_info_map,
        target_port: channel.target_port.clone(),
        security_state: channel.security_state,
        proxy_tcp,
        client_tcp: npc_pool_tcp,
        channel_data_total: data_total,
    };
    tokio::spawn(bridge.start());
}

// 关闭客户端所有正在通信的连接
fn ShutdownByClient(client_id: u8) {
    // bridgeLock.Lock()

    // //帅选出要删除的客户端桥接
    // for bridge := range bridgeMap {
    // 	if bridge.ClientId == client_id {
    // 		bridge.shutdown()
    // 	}
    // }
    // bridgeLock.Unlock()
}

// 关闭隧道所有正在通信的连接
fn ShutdownByChannel(channel_id: u8) {
    // bridgeLock.Lock()

    // //帅选出要删除的客户端桥接
    // for bridge := range bridgeMap {
    // 	if bridge.Channel.Id == channelId {
    // 		bridge.shutdown()
    // 	}
    // }
    // bridgeLock.Unlock()
}

// 移除桥接通信
fn removeBridge(bridge: TCPBridge) {
    // bridgeLock.Lock()
    // delete(bridgeMap, bridge)
    // bridgeLock.Unlock()
}

/**
 * 回收长时间不用的连接
 */
fn Recycle() {
    //while (true) {
    //    delay(CLSConfig.BRIDGE_SESSION_TIMEOUT)
    //    try {
    //
    //        //当前是同时间戳
    //        val now = System.currentTimeMillis()
    //    result: List<TCPBridge>? = null
    //        this.BridgeListLock.synchronized {
    //            result = this.BridgeList.filter {
    //                (now - it.lastSessionTime) > CLSConfig.BRIDGE_SESSION_TIMEOUT
    //            }
    //        }
    //        result?.forEach { //关掉长时间不通信的连接
    //            it.close()
    //        }
    //    } catch (e: Exception) {
    //        //e.printStackTrace()
    //    }
    //}
}
