use crate::nps::security_util;
use crate::nps_error::NpsError;
use bytes::Bytes;
use np_common::{head_flag, time_util};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Receiver;
use tokio::sync::Notify;
use tokio::select;

/**
 * 服务端与客户端通信类
 * @param client 客户端DTO
 * @param clientSocket 与客户端的连接
 */
// pub struct ClientSession {
//     // pub session_tag: &'a str,//会话标识,由当前时间戳和随机数生成,保证唯一性
//     pub client: Client,
//     pub tcp: TcpStream,
//
//     //最后一次收到客户端心跳时间
//     pub heart_time: Arc<AtomicU64>,
// }

// impl ClientSession {
// 开始会话
pub async fn start(
    client_id: i64,
    mut npc_tcp: TcpStream,
    closer: Arc<Notify>,
    heart_time: Arc<AtomicU64>,
    mut external_rx: Receiver<Bytes>,
) -> Result<(), NpsError> {

    //将客户端id发送给NPC客户端
    npc_tcp.write_i64(client_id).await?;

    // 将加密秘钥发送到客户端
    npc_tcp
        .write_all(&*security_util::CLIENT_SECURITY_KEY)
        .await?;
    //
    // //拆分读写流,接收和发送分开处理
    // let (mut reader1, mut writer1) = io::split(&mut self.tcp);
    // let write1 = &mut writer1;
    //
    // let writer_lock = Mutex::new(writer1);
    //
    // //开启一个异步任务,专门负责接收从客户端发来的数据,并将数据写入到writer中
    // let (tx1, mut rx1) = tokio::sync::mpsc::channel::<Bytes>(1024);

    // //开启一个异步任务,专门负责将从其他地方发送过来的数据写入到客户端连接中
    // let external_write_task = async move {
    //     while let Some(bytes) = rx.recv().await {
    //         if let Err(e) = writer.write_all(bytes.as_ref()).await {
    //             return Err(NpsError::IoError(e));
    //         }
    //     }
    //     writer.shutdown().await?;
    //     Ok(())
    // };

    // //开启一个异步任务,专门负责将从其他地方发送过来的数据写入到客户端连接中
    // let external_receive_task = async {
    //     while let Some(bytes) = external_rx.recv().await {
    //         writer_lock.lock().await.write_all(&*bytes).await?;
    //         // writer1.write_all(bytes.as_ref()).await?;
    //
    //         // if let Err(_) = tx.send(bytes).await {
    //         //     //对方可能已经关闭，直接结束
    //         //     return Err(NpsError::SendDataError);
    //         // }
    //     }
    //     Ok(())
    // };

    // let heart_time = &self.heart_time;
    // let receive_task = async move {
    //     loop {
    //         select! {
    //             recv_result = external_rx.recv() => {
    //                 let Some(bytes) = recv_result else {
    //
    //                     //对方可能已经关闭，直接结束
    //                     return Err(NpsError::SendDataError);
    //                 }
    //                 writer_lock.lock().await.write_all(&*bytes).await?;
    //             }
    //             flag = reader1.read_u8() =>{//从npc客户端读取标记
    //                 let flag = flag?;
    //                 if flag != head_flag::MAIN_HEART_BEAT {
    //                     return Err(NpsError::UnknowFlagError(flag));
    //                 }
    //                 writer_lock
    //                     .lock()
    //                     .await
    //                     .write_u8(head_flag::MAIN_HEART_BEAT)
    //                     .await?;
    //
    //                 //记录当前心跳时间
    //                 heart_time.store(time_util::current_millis(), Ordering::Relaxed);
    //             }
    //         }
    //     }
    //     Ok(())
    // };
    // select! {
    //     _ = closer.notified() => {
    //
    //         //这里必须要关闭,否则客户端无法感知对方已经关闭
    //         writer_lock.lock().await.shutdown().await?;
    //         Ok(())
    //     }
    //     result = async {try_join!(receive_task)} =>{
    //         result?;
    //         Ok(())
    //     }
    // }
    let notified = closer.notified();
    tokio::pin!(notified);
    loop {
        select! {
            _ = &mut notified => {
                println!("-->收到关闭客户端通知");
                break;
            }
            recv_result = external_rx.recv() => {
                let Some(bytes) = recv_result else {

                    //对方可能已经关闭，直接结束
                    return Err(NpsError::SendDataError);
                };
                npc_tcp.write_all(&*bytes).await?;
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

// async fn receive_data(
//     mut external_rx: Receiver<Bytes>,
//     heart_time: Arc<AtomicU64>,
//     mut npc_tcp: TcpStream,
//     closer: Arc<Notify>,
// ) -> Result<(), NpsError> {
//     let notified = closer.notified();
//     tokio::pin!(notified);
//     loop {
//         select! {
//             _ = &mut notified => {
//                 break;
//             }
//             recv_result = external_rx.recv() => {
//                 let Some(bytes) = recv_result else {
//
//                     //对方可能已经关闭，直接结束
//                     return Err(NpsError::SendDataError);
//                 }
//                 npc_tcp.write_all(&*bytes).await?;
//             }
//             flag = npc_tcp.read_u8() =>{//从npc客户端读取标记
//                 let flag = flag?;
//                 if flag != head_flag::MAIN_HEART_BEAT {
//                     return Err(NpsError::UnknowFlagError(flag));
//                 }
//                 npc_tcp.write_u8(head_flag::MAIN_HEART_BEAT).await?;
//
//                 //记录当前心跳时间
//                 heart_time.store(time_util::current_millis(), Ordering::Relaxed);
//             }
//         }
//     }
//     Ok(())
// }

// /**
//  * 处理从客户端收到的消息
//  */
// async fn handle(
//     heart_time: &AtomicU64,
//     writer: &mut WriteHalf<&mut TcpStream>,
//     flag: u8,
// ) -> Result<(), NpsError> {
//     match flag {
//         head_flag::MAIN_HEART_BEAT => {
//             writer.write_u8(head_flag::MAIN_HEART_BEAT).await?;
//             // if let Err(_) = tx
//             //     .send(Bytes::from_static(&[head_flag::MAIN_HEART_BEAT]))
//             //     .await
//             // {
//             //     return Err(NpsError::SendDataError);
//             // }
//
//             //记录当前心跳时间
//             heart_time.store(time_util::current_millis(), Ordering::Relaxed);
//             Ok(())
//         }
//         //这里抛出Error
//         _ => Err(NpsError::UnknowFlagError(flag)),
//     }
// }
// }
