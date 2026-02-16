// package tcp_client
//
// import (
//     "DairoNPS/dao/dto"
//     "DairoNPS/nps/nps_client/HeaderUtil"
//     "DairoNPS/nps/nps_pool/tcp_pool"
//     "DairoNPS/nps/nps_pool/udp_pool"
//     "DairoNPS/nps/nps_proxy/tcp_proxy"
//     "DairoNPS/nps/nps_proxy/udp_proxy"
//     "net"
//     "strconv"
//     "sync"
// )

use super::tcp_client_session::ClientSession;
use crate::entity::client::Client;
use crate::nps::nps_client::header_util;
use bytes::{BufMut, Bytes, BytesMut};
use std::collections::HashMap;
use std::sync::OnceLock;
use tokio::io;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;

// type ClientSessionManager struct{}

//往客户端发送指令的专用连接

/**
 * 客户端ID对应的Socket连接
 */
static CLIENT_SESSION_MAP: OnceLock<Mutex<HashMap<u64, Sender<Bytes>>>> = OnceLock::new();
//
// /**
//  * 添加互斥锁
//  */
// var clientSessionLock sync.Mutex
//
// // 保持客户端连接
pub async fn hold_on_client(client: Client, tcp: TcpStream) -> io::Result<()> {
    let mut session_map = CLIENT_SESSION_MAP
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
        .await;

    //先移除之前的连接
    // clientSessionLock.Lock()
    // oldSession := clientSessionMap[client.Id]
    // clientSessionLock.Unlock()
    // if oldSession != nil { //如果存在
    //     oldSession.Shutdown()
    // }

    let client_id = client.id;
    if let Some(old_session_tx) = session_map.remove(&client_id) {
        println!("关闭之前的连接:{:?}", old_session_tx);

        // 发送关闭指令
        if let Err(e) = old_session_tx
            .send(Bytes::from_static(header_util::CLOSE_CMD))
            .await
        {
            return io::Result::Err(io::Error::new(
                io::ErrorKind::Other,
                format!("发送关闭指令失败:{}", e),
            ));
        }
    }

    let (tx, rx) = tokio::sync::mpsc::channel::<Bytes>(1024);
    session_map.insert(client_id, tx);
    drop(session_map);

    // //新的回话
    let mut session = ClientSession {
        client,
        tcp,
        last_heart_beat_time: 0,
    };
    // clientSessionLock.Lock()
    // clientSessionMap[client.Id] = session
    // clientSessionLock.Unlock()
    //
    // //初始化客户端连接池
    // tcp_pool.InitEmptyPoolByClient(client.Id)
    // udp_pool.InitEmptyPoolByClient(client.Id)
    //
    // //开启该客户端下所有隧道监听
    // tcp_proxy.AcceptClient(client)
    // udp_proxy.AcceptClient(client)
    let rs = session.start(rx).await;
    CLIENT_SESSION_MAP
        .get()
        .unwrap()
        .lock()
        .await
        .remove(&client_id);
    rs?;
    Ok(())
}

/**
 * 向客户端申请TCP连接池请求
 * @param clientId 客户端ID
 * @param count 申请数量
 */
pub async fn send_tcp_pool_request(client_id: u64, count: i32) {
    send(client_id, header_util::REQUEST_TCP_POOL, count.to_string().as_str()).await;
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
    let session_map = CLIENT_SESSION_MAP.get().unwrap().lock().await;
    let tx_option = session_map.get(&client_id).cloned();
    drop(session_map); // 释放锁
    if let Some(tx) = tx_option {
        let mut bm = BytesMut::with_capacity(1 + message.as_bytes().len());
        bm.put_u8(flag);
        bm.put_slice(message.as_bytes());
        tx.send(bm.freeze()).await.unwrap();
    }
}

// // 关闭客户端
// // - closeSession 当前关闭的对象
// func removeSession(closeSession *ClientSession) {
//     shutdownProxyAndPoolAndBridge(closeSession.Client.Id)
//     clientId := closeSession.Client.Id
//     clientSessionLock.Lock()
//     session := clientSessionMap[clientId]
//     if session != nil { //客户端ID回话如果存在
//         if session == closeSession { //当前没有加入新的回话
//             delete(clientSessionMap, clientId)
//         } else { //由于关闭延迟,有新的回话加入,但是在之前已经关掉了所有的代理监听,所以这里需要再次开启代理监听,概率很小，但不能排除
//             go tcp_proxy.AcceptClient(session.Client)
//             go udp_proxy.AcceptClient(session.Client)
//         }
//     }
//     clientSessionLock.Unlock()
// }
//
// // 关闭与内网穿透客户端的会话连接
// func shutdownProxyAndPoolAndBridge(clientId int) {
//
//     //关闭代理监听
//     tcp_proxy.ShutdownByClient(clientId)
//     udp_proxy.ShutdownByClient(clientId)
//
//     //关闭所有连接池
//     tcp_pool.ShutdownByClient(clientId)
//     udp_pool.ShutdownByClient(clientId)
//
//     //关闭所有UDP连接池
//     //try {
//     //   UDPPoolManager.closeByClient(this.client.id!!)
//     //} catch (e: Exception) {
//     //   e.printStackTrace()
//     //}
//     //
//     //try {
//     //   //关闭正在通信的UDP连接
//     //   UDPBridgeManager.closeByClient(this.client.id!!)
//     //} catch (e: Exception) {
//     //   e.printStackTrace()
//     //}
// }
//
// // 关闭一个客户端
// func Shutdown(clientId int) {
//
//     //先移除之前的连接
//     clientSessionLock.Lock()
//     oldSession := clientSessionMap[clientId]
//     clientSessionLock.Unlock()
//     if oldSession != nil { //如果存在
//         oldSession.Shutdown()
//     }
// }
//
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
