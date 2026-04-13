use super::tcp_client_session::ClientSession;
use crate::dao::client_dao::Client;
use crate::nps;
use crate::nps::ClientLive;
use crate::nps::nps_client::header_util;
use crate::nps::nps_proxy::tcp_proxy_manager;
use bytes::Bytes;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use tokio::io;
use tokio::net::TcpStream;
use tokio::sync::Notify;
use tokio::time::Duration;
use crate::nps_error::NpsError;
use crate::util::time_util;
//往客户端发送指令的专用连接

// 保持客户端连接
pub async fn hold_on_client(client: Client, tcp: TcpStream) -> Result<(),NpsError> {
    let client_id = client.id;

    // 先尝试关闭之前的连接
    shutdown(client_id).await?;

    //用来记录最后一次心跳时间
    let heart_time = Arc::new(AtomicU64::new(time_util::current_millis()));
    let (tx, rx) = tokio::sync::mpsc::channel::<Bytes>(1024);
    nps::CLIENT_LIVE_MAP.lock().await.insert(
        client_id,
        ClientLive {
            tcp_pool: Vec::new(),
            sender: tx,
            heart_time: heart_time.clone(),
        },
    );

    // //新的回话
    let mut session = ClientSession {
        client,
        tcp,
        heart_time,
    };

    //初始化客户端连接池
    // udp_pool.InitEmptyPoolByClient(client.Id)
    //
    // //开启该客户端下所有隧道监听
    tcp_proxy_manager::accept_client(client_id).await?;
    // udp_proxy.AcceptClient(client)
    let rs = session.start(rx).await;

    //会话结束后,移除会话
    remove_session(client_id).await;
    rs
}

/**
 * 向客户端申请TCP连接池请求
 * @param clientId 客户端ID
 * @param count 申请数量
 */
pub async fn send_tcp_pool_request(client_id: i64, count: u8) {
    send(
        client_id,
        header_util::REQUEST_TCP_POOL,
        count.to_string().as_str(),
    )
    .await;
}
//
// /**
//  * 向客户端申请UDP连接池请求
//  * @param clientID 客户端ID
//  * @param count 申请数量
//  */
// fn SendUDPPoolRequest(client_id:u64, count: i32) {
//     send(clientId, HeaderUtil.REQUEST_UDP_POOL, strconv.Itoa(count))
// }
//
// /**
//  * 向客户端当前激活的UDP端口
//  * @param clientID 客户端ID
//  * @param count 申请数量
//  */
// func (mine *ClientSessionManager) SendActiveUDPBridge(clientId int, ports string) {
//     send(clientId, HeaderUtil.SYNC_ACTIVE_BRIDGE_UDP_PORT, ports)
// }

/**
 * 往客户端发送数据
 * @param clientID 客户端ID
 * @param flag 头部标记
 * @param message 头部消息
 */
async fn send(client_id: i64, flag: u8, message: &str) {
    let tx = {
        if let Some(client_nps) = nps::CLIENT_LIVE_MAP.lock().await.get(&client_id) {
            client_nps.sender.clone()
        } else {
            return;
        }
    };
    let header_data = header_util::make_header_data(flag, message);
    tx.send(header_data)
        .await
        .unwrap_or_else(|it| println!("-->往客户端发送数据失败:{:?}", it));
}

// 关闭客户端
// - closeSession 当前关闭的对象
pub async fn remove_session(client_id: i64) {
    // shutdown_proxy_and_pool_and_bridge(client_id).await;

    //移除连接
    nps::CLIENT_LIVE_MAP.lock().await.remove(&client_id);

    //获取当前客户端下的所有隧道关闭监听器
    let channel_ids: Vec<Arc<Notify>> = nps::CHANNEL_LIVE_MAP
        .lock()
        .await
        .iter()
        .filter(|(channel_id, channel_nps)| channel_nps.client_id == client_id)
        .map(|(channel_id, channel_nps)| channel_nps.closer.clone())
        .collect();

    // 关闭正在监听的隧道
    channel_ids.iter().for_each(|it| it.notify_waiters());
}

// // 关闭与内网穿透客户端的会话连接
// async fn shutdown_proxy_and_pool_and_bridge(client_id: i64) {
// //关闭代理监听
// tcp_proxy.ShutdownByClient(clientId)
// udp_proxy.ShutdownByClient(clientId)

//关闭所有连接池
// tcp_pool_manager::shutdown_by_client(client_id).await;
// udp_pool.ShutdownByClient(clientId)

//关闭所有UDP连接池
//try {
//   UDPPoolManager.closeByClient(this.client.id!!)
//} catch (e: Exception) {
//   e.printStackTrace()
//}
//
//try {
//   //关闭正在通信的UDP连接
//   UDPBridgeManager.closeByClient(this.client.id!!)
//} catch (e: Exception) {
//   e.printStackTrace()
//}
// }

// 关闭一个客户端
pub async fn shutdown(client_id: i64) -> io::Result<()> {
    let tx = {
        if let Some(client_nps) = nps::CLIENT_LIVE_MAP.lock().await.get(&client_id) {
            client_nps.sender.clone()
        } else {
            return Ok(());
        }
    };

    // 发送关闭指令
    if let Err(e) = tx.send(Bytes::from_static(header_util::CLOSE_CMD)).await {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("发送关闭指令失败:{}", e),
        ));
    }

    // 等待一段时间让旧连接关闭
    while nps::CLIENT_LIVE_MAP.lock().await.contains_key(&client_id) {
        //这里很快就会被关闭，所以不需要设置过长的等待时间
        // println!("-->等待旧连接关闭...");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    Ok(())
}

// // 客户端是否在线监测
// func IsOnline(clientId int) bool {
//     clientSessionLock.Lock()
//     session := clientSessionMap[clientId]
//     clientSessionLock.Unlock()
//     if session == nil {
//         return false
//     }
//     return session.IsOnline()
// }
//
// // 获取当前在线客户端数量
// func OnlineCount() int {
//     onlineClientCount := 0
//     clientSessionLock.Lock()
//     for _, session := range clientSessionMap {
//         if session.IsOnline() {
//             onlineClientCount++
//         }
//     }
//     clientSessionLock.Unlock()
//     return onlineClientCount
// }
