use super::tcp_client_session::ClientSession;
use crate::dao::client_dao::Client;
use crate::nps;
use crate::nps::ClientLive;
use crate::nps::nps_client::header_util;
use crate::nps::nps_proxy::tcp_proxy;
use crate::nps_error::NpsError;
use crate::util::time_util;
use bytes::{BufMut, Bytes, BytesMut};
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use tokio::io;
use tokio::net::TcpStream;
use tokio::sync::Notify;
use tokio::time::Duration;
//往客户端发送指令的专用连接

// 保持客户端连接
pub async fn hold_on_client(client: Client, tcp: TcpStream) -> Result<(), NpsError> {
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

    // //开启该客户端下所有隧道监听
    tcp_proxy::ready_client(client_id).await?;
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
    let tx = {
        if let Some(client_live) = nps::CLIENT_LIVE_MAP.lock().await.get(&client_id) {
            client_live.sender.clone()
        } else {
            return;
        }
    };
    tx.send(Bytes::from(vec![header_util::REQUEST_TCP_POOL, count]))
        .await
        .unwrap_or_else(|it| println!("-->往客户端发送数据失败:{:?}", it));
}

// /**
//  * 往客户端发送数据
//  * @param clientID 客户端ID
//  * @param flag 头部标记
//  * @param message 头部消息
//  */
// async fn send(client_id: i64, flag: u8, message: &str) {
//     let tx = {
//         if let Some(client_live) = nps::CLIENT_LIVE_MAP.lock().await.get(&client_id) {
//             client_live.sender.clone()
//         } else {
//             return;
//         }
//     };
//     let header_data = header_util::make_header_data(flag, message);
//     tx.send(header_data)
//         .await
//         .unwrap_or_else(|it| println!("-->往客户端发送数据失败:{:?}", it));
// }

// 关闭客户端
pub async fn remove_session(client_id: i64) {
    //移除连接
    nps::CLIENT_LIVE_MAP.lock().await.remove(&client_id);

    //获取当前客户端下的所有隧道关闭监听器
    let channel_ids: Vec<Arc<Notify>> = nps::CHANNEL_LIVE_MAP
        .lock()
        .await
        .iter()
        .filter(|(channel_id, channel_live)| channel_live.client_id == client_id)
        .map(|(channel_id, channel_live)| channel_live.closer.clone())
        .collect();

    // 关闭正在监听的隧道
    channel_ids.iter().for_each(|it| it.notify_waiters());
}

// 关闭一个客户端
pub async fn shutdown(client_id: i64) -> io::Result<()> {
    let tx = {
        if let Some(client_live) = nps::CLIENT_LIVE_MAP.lock().await.get(&client_id) {
            client_live.sender.clone()
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
