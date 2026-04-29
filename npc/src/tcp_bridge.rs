use crate::npc_error::NpcError;
use std::sync::atomic::Ordering;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::{io, select};
use crate::application;

pub struct TCPBridgeParam {
    //数据是否加密
    pub is_encode_data: bool,

    //与NPS服务端的TCP
    pub nps_tcp: TcpStream,

    //目标服务器的地址
    pub target_addr: String,
}

/**
 * 开始会话
 * @param client 客户端DTO
 * @param channel 隧道信息
 * @param proxySocket 代理服务端Socket
 * @param clientSocket 内网穿透客户端Socket
 */
pub fn ready(param: TCPBridgeParam) {
    tokio::spawn(async move {
        //桥接数+1
        application::BRIDGE_COUNT.fetch_add(1, Ordering::Relaxed);

        spawn_start(param).await;

        //桥接数-1
        application::BRIDGE_COUNT.fetch_sub(1, Ordering::Relaxed);
    });
}

/// 异步启动桥接通信，并监听关闭通知
async fn spawn_start(param: TCPBridgeParam) {
    //npc服务端关闭通知
    let closer = application::NPC_CLOSER.load();
    select! {
        _ = closer.notified() => {
            println!("收到关闭通知，准备关闭桥接通信...");
            return;
        }
        result = start(param) => {
            if let Err(e) = result {
                println!("桥接通信接发生了错误:{:?}", e);
            }
        }
    }
}

/// TCP桥接通信开始
async fn start(mut param: TCPBridgeParam) -> Result<(), NpcError> {
    // 建立连接
    let mut target_tcp = match TcpStream::connect(param.target_addr).await {
        Ok(v) => v,
        Err(e) => {
            //与目标服务器连接失败时，直接关闭
            let _ = param.nps_tcp.shutdown().await;
            return Err(NpcError::IoError(e));
        }
    };
    if let Err(e) = io::copy_bidirectional(&mut target_tcp, &mut param.nps_tcp).await {
        return Err(NpcError::IoError(e));
    }
    Ok(())
}
