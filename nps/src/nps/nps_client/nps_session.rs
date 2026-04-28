use crate::dao::client_dao::Client;
use crate::nps;
use crate::nps::nps_proxy::tcp_proxy;
use crate::nps::{security_util, ClientLive};
use crate::nps_error::NpsError;
use np_common::{head_flag, time_util};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Notify;
use tokio::time::Duration;
use tokio::{io, select};
//往客户端发送指令的专用连接

// 保持客户端连接
pub async fn hold_on(client: Client, npc_tcp: TcpStream) -> Result<(), NpsError> {
    let client_id = client.id;

    // 先尝试关闭之前的连接
    shutdown(client_id).await?;

    //用来记录最后一次心跳时间
    let heart_time = Arc::new(AtomicU64::new(time_util::current_millis()));
    let (sender, receiver) = tokio::sync::mpsc::channel::<Vec<u8>>(1024);
    let closer = Arc::new(Notify::new());
    nps::CLIENT_LIVE_MAP.lock().await.insert(
        client_id,
        ClientLive {
            tcp_pool: Vec::new(),
            sender,
            heart_time: heart_time.clone(),
            closer: closer.clone(),
        },
    );

    // //开启该客户端下所有隧道监听
    tcp_proxy::ready_client(client_id).await?;
    let rs = start(client_id, npc_tcp, closer, heart_time, receiver).await;

    //会话结束后,移除会话
    remove_session(client_id).await;
    rs
}

/// 开始会话
async fn start(
    client_id: i64,
    mut npc_tcp: TcpStream,
    closer: Arc<Notify>,
    heart_time: Arc<AtomicU64>,
    mut receiver: Receiver<Vec<u8>>,
) -> Result<(), NpsError> {
    //将客户端id发送给NPC客户端
    npc_tcp.write_i64(client_id).await?;

    // 将加密秘钥发送到客户端
    npc_tcp
        .write_all(&*security_util::CLIENT_SECURITY_KEY)
        .await?;
    let notified = closer.notified();
    tokio::pin!(notified);
    loop {
        select! {
            _ = &mut notified => {
                println!("-->收到关闭客户端通知");
                break;
            }
            recv_result = receiver.recv() => {
                let Some(data) = recv_result else {

                    //对方可能已经关闭，直接结束
                    return Err(NpsError::SendDataError);
                };
                npc_tcp.write_all(&data).await?;
            }
            flag = npc_tcp.read_u8() =>{//从npc客户端读取标记
                let flag = flag?;
                if flag != head_flag::MAIN_HEART_BEAT {
                    return Err(NpsError::UnknowFlagError(flag));
                }
                npc_tcp.write_u8(head_flag::MAIN_HEART_BEAT).await?;

                //记录当前心跳时间
                heart_time.store(time_util::current_millis(), Ordering::Relaxed);
            }
        }
    }
    Ok(())
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
    tx.send(vec![head_flag::REQUEST_TCP_POOL, count])
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
async fn remove_session(client_id: i64) {
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
    // let tx = {
    //     if let Some(client_live) = nps::CLIENT_LIVE_MAP.lock().await.get(&client_id) {
    //         client_live.sender.clone()
    //     } else {
    //         return Ok(());
    //     }
    // };
    //
    // // 发送关闭指令
    // if let Err(e) = tx.send(Bytes::from_static(head_flag::CLOSE_CMD)).await {
    //     return Err(io::Error::new(
    //         io::ErrorKind::Other,
    //         format!("发送关闭指令失败:{}", e),
    //     ));
    // }
    //
    // // 等待一段时间让旧连接关闭
    // while nps::CLIENT_LIVE_MAP.lock().await.contains_key(&client_id) {
    //     //这里很快就会被关闭，所以不需要设置过长的等待时间
    //     // println!("-->等待旧连接关闭...");
    //     tokio::time::sleep(Duration::from_millis(10)).await;
    // }
    loop {
        let client_map = nps::CLIENT_LIVE_MAP.lock().await;
        if let Some(client_live) = client_map.get(&client_id) {
            client_live.closer.notify_waiters();
            drop(client_map);

            //这里很快就会被关闭，所以不需要设置过长的等待时间
            // println!("-->等待旧连接关闭...");
            tokio::time::sleep(Duration::from_millis(10)).await;
            continue;
        }
        break;
    }

    Ok(())
}
