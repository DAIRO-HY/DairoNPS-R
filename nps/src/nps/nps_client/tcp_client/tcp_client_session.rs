use crate::dao::client_dao::Client;
use crate::nps::nps_client::header_util;
use crate::nps::nps_error::NpsError;
use crate::util::{security_util, time_util};
use bytes::Bytes;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::{io, try_join};

/**
 * 服务端与客户端通信类
 * @param client 客户端DTO
 * @param clientSocket 与客户端的连接
 */
pub struct ClientSession {
    // pub session_tag: &'a str,//会话标识,由当前时间戳和随机数生成,保证唯一性
    pub client: Client,
    pub tcp: TcpStream,

    //最后一次收到客户端心跳时间
    pub heart_time: Arc<AtomicU64>,
}

/**
 * 发送数据互斥锁
 */
// var sendLock sync.Mutex

impl ClientSession {
    // 开始会话
    pub async fn start(&mut self, external_rx: Receiver<Bytes>) -> Result<(), NpsError> {
        // self.receive().await;
        self.send_server_info_and_receive(external_rx).await
        //time.Sleep(1 * time.Second)
        // mine.Shutdown()
        // removeSession(mine)
    }

    // 发送服务器端的信息
    async fn send_server_info_and_receive(
        &mut self,
        mut external_rx: Receiver<Bytes>,
    ) -> Result<(), NpsError> {
        let tcp_stream = &mut self.tcp;

        //将客户端id发送给NPC客户端
        let header_data = header_util::make_header_data(
            header_util::SERVER_TO_CLIENT_ID,
            &self.client.id.to_string(),
        );
        tcp_stream.write_all(header_data.as_ref()).await?;

        // 将加密秘钥发送到客户端
        tcp_stream
            .write_all(&*security_util::CLIENT_SECURITY_KEY)
            .await?;

        //拆分读写流,接收和发送分开处理
        let (mut reader, mut writer) = tokio::io::split(&mut self.tcp);

        //开启一个异步任务,专门负责接收从客户端发来的数据,并将数据写入到writer中
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Bytes>(1024);

        //开启一个异步任务,专门负责将从其他地方发送过来的数据写入到客户端连接中
        let external_write_task = async move {
            while let Some(bytes) = rx.recv().await {
                if bytes.len() == header_util::CLOSE_CMD.len()
                    && bytes.as_ref() == header_util::CLOSE_CMD
                {
                    //如果收到关闭指令,则关闭连接
                    break;
                }
                if let Err(e) = writer.write_all(bytes.as_ref()).await{
                    return Err(NpsError::IoError(e))
                }
            }
            writer.shutdown().await?;
            Ok(())
        };

        //开启一个异步任务,专门负责将从其他地方发送过来的数据写入到客户端连接中
        let external_receive_task = {
            let tx = tx.clone();
            async move {
                while let Some(bytes) = external_rx.recv().await {
                    if let Err(_) = tx.send(bytes).await {
                        //对方可能已经关闭，直接结束
                        return Err(NpsError::SendDataError)
                    }
                }
                Ok(())
            }
        };

        let heart_time = &self.heart_time;
        let receive_task = async move {
            loop {
                let mut flag_data = [0u8; 1];
                if let Err(e) = reader.read_exact(&mut flag_data).await{
                    return Err(NpsError::IoError(e));
                }
                Self::handle(heart_time, &tx, flag_data[0]).await?;
            }
            Ok(())
        };
        try_join!(external_write_task, external_receive_task, receive_task)?;
        Ok(())
    }

    // /**
    //  * 接收从客户端发来的数据
    //  */
    // async fn receive(
    //     heart_time: &AtomicU64,
    //     mut reader: ReadHalf<&mut TcpStream>,
    //     tx: &Sender<Bytes>,
    // ) -> io::Result<()> {
    //     loop {
    //         let mut flag_data = [0u8; 1];
    //         reader.read_exact(&mut flag_data).await?;
    //         Self::handle(heart_time, tx, flag_data[0]).await?;
    //     }
    // }

    /**
     * 处理从客户端收到的消息
     */
    async fn handle(heart_time: &AtomicU64, tx: &Sender<Bytes>, flag: u8) -> Result<(), NpsError> {
        match flag {
            header_util::MAIN_HEART_BEAT => {
                if let Err(_) = tx.send(Bytes::from_static(&[header_util::MAIN_HEART_BEAT]))
                    .await{
                    return Err(NpsError::SendDataError)
                }

                //记录当前心跳时间
                heart_time.store(
                    time_util::current_millis(),
                    Ordering::Relaxed,
                );
                Ok(())
            }
            //这里抛出Error
            _ => Err(NpsError::UnknowFlagError),
        }
    }

    // /**
    //  * 往客户端发送数据
    //  * @param flag 头部标记
    //  * @param message 头部消息
    //  */
    // async fn send_head(tcp_stream: &mut TcpStream, flag: u8, message: &str) -> io::Result<()> {
    //     tcp_stream.write_all(&[flag, message.len() as u8]).await?;
    //     tcp_stream.write_all(message.as_bytes()).await?;
    //     Ok(())
    // }

    // /**
    //  * 往客户端发送数据
    //  * @param data 要发送的数据
    //  */
    // fn Send(data []uint8) {
    //     err := WriterUtil.WriteFull(mine.tcp, data)
    // }

    // /**
    //  * 关闭与内网穿透客户端的会话连接
    //  */
    // func (mine *ClientSession) Shutdown() {
    //     mine.tcp.Close()
    // }
    //
    // // 客户端是否在线监测
    // func (mine *ClientSession) IsOnline() bool {
    //     now := time.Now().UnixMilli()
    //
    //     //在指定时间内没有收到客户端心跳,则视为离线
    //     if now-mine.lastHeartBeatTime > NPSConstant.HEART_TIME*2 {
    //         return false
    //     }
    //     return true
    // }
}
