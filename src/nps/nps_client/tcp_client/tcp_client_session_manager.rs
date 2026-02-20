use super::tcp_client_session::ClientSession;
use crate::entity::client::Client;
use crate::nps::nps_client::header_util;
use crate::nps::nps_pool::tcp_pool_manager;
use crate::nps::nps_proxy::tcp_proxy_manager;
use bytes::{BufMut, Bytes, BytesMut};
use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::io;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;
use crate::nps::CLIENT_SESSION;
// type ClientSessionManager struct{}

//往客户端发送指令的专用连接


// 保持客户端连接
pub async fn hold_on_client(client: Client, tcp: TcpStream) -> io::Result<()> {
    println!("-->新连接...");
    let client_id = client.id;

    // 先尝试关闭之前的连接
    shutdown(client_id).await?;
    let (tx, rx) = tokio::sync::mpsc::channel::<Bytes>(1024);
    CLIENT_SESSION
        .get()
        .unwrap()
        .lock()
        .await
        .insert(client_id, tx);

    // //新的回话
    let mut session = ClientSession {
        client,
        tcp,
        last_heart_beat_time: 0,
    };

    //初始化客户端连接池
    tcp_pool_manager::init_empty_pool_by_client(client_id).await;
    // udp_pool.InitEmptyPoolByClient(client.Id)
    //
    // //开启该客户端下所有隧道监听
    tcp_proxy_manager::accept_client(client_id).await;
    // udp_proxy.AcceptClient(client)
    let rs = session.start(rx).await;

    //会话结束后,移除会话
    remove_session(client_id).await;
    rs?;
    Ok(())
}

/**
 * 向客户端申请TCP连接池请求
 * @param clientId 客户端ID
 * @param count 申请数量
 */
pub async fn send_tcp_pool_request(client_id: u64, count: u8) {
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
async fn send(client_id: u64, flag: u8, message: &str) {
    let tx_option = {
        let session_map = CLIENT_SESSION.get().unwrap().lock().await;
        session_map.get(&client_id).cloned()
    };
    if let Some(tx) = tx_option {
        let header_data = header_util::make_header_data(flag, message);
        tx.send(header_data).await.unwrap();
    }
}

// 关闭客户端
// - closeSession 当前关闭的对象
pub async fn remove_session(client_id: u64) {
    shutdown_proxy_and_pool_and_bridge(client_id).await;

    //移除连接
    CLIENT_SESSION
        .get()
        .unwrap()
        .lock()
        .await
        .remove(&client_id);
}

// 关闭与内网穿透客户端的会话连接
async fn shutdown_proxy_and_pool_and_bridge(client_id: u64) {
    // //关闭代理监听
    // tcp_proxy.ShutdownByClient(clientId)
    // udp_proxy.ShutdownByClient(clientId)

    //关闭所有连接池
    tcp_pool_manager::shutdown_by_client(client_id).await;
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
}

// 关闭一个客户端
pub async fn shutdown(client_id: u64) -> io::Result<()> {
    let old_session_tx = {
        let session_map = CLIENT_SESSION.get().unwrap().lock().await;
        session_map.get(&client_id).cloned()
    };
    if let Some(tx) = old_session_tx {
        //如果存在旧的连接

        // 发送关闭指令
        if let Err(e) = tx.send(Bytes::from_static(header_util::CLOSE_CMD)).await {
            return io::Result::Err(io::Error::new(
                io::ErrorKind::Other,
                format!("发送关闭指令失败:{}", e),
            ));
        }

        // 等待一段时间让旧连接关闭
        while CLIENT_SESSION
            .get()
            .unwrap()
            .lock()
            .await
            .contains_key(&client_id)
        {
            //这里很快就会被关闭，所以不需要设置过长的等待时间
            println!("-->等待旧连接关闭...");
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
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
