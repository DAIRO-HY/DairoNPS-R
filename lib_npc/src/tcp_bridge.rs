use std::sync::Arc;
use crate::application;
use crate::npc_error::NpcError;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::{io, select, try_join};
use lib_np_common::data_io_len::AtomicDataIOLen;
use lib_np_common::time_util;

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
    //npc服务端关闭通知
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
    let closer = &application::NPC_CLOSER;
    select! {
        _ = closer.notified() => {
            // println!("收到关闭通知，准备关闭桥接通信...");
            return;
        }
        result = start(param) => {
            if let Err(e) = result {
                println!("-->桥接通信接发生了错误:{:?}", e);
            }
        }
    }
}

/// TCP桥接通信开始
async fn start(mut param: TCPBridgeParam) -> Result<(), NpcError> {
    // 建立连接
    let target_tcp = match TcpStream::connect(param.target_addr).await {
        Ok(v) => v,
        Err(e) => {
            //与目标服务器连接失败时，直接关闭
            let _ = param.nps_tcp.shutdown().await;
            return Err(NpcError::IoError(e));
        }
    };
    let (nps_reader, nps_writer) = tokio::io::split(param.nps_tcp);
    let (target_reader, target_writer) = tokio::io::split(target_tcp);
    if param.is_encode_data { //如果需要加密传输
        
        //这里不能使用&*application::SECURITY_KEY.read().await来获取指针,这会导致锁无法被释放
        let security_keys = &application::SECURITY_KEY.read().await.clone();
        let nps2target = encode_traffic(nps_reader, target_writer, security_keys, &application::OUT_LEN);
        let target2nps = encode_traffic(target_reader, nps_writer, security_keys, &application::IN_LEN);
        try_join!(nps2target, target2nps)?;
    } else {
        let nps2target = traffic(nps_reader, target_writer, &application::OUT_LEN);
        let target2nps = traffic(target_reader, nps_writer, &application::IN_LEN);
        try_join!(nps2target, target2nps)?;
    }
    Ok(())
}

/// 加密传输
async fn encode_traffic(
    mut reader: ReadHalf<TcpStream>,
    mut writer: WriteHalf<TcpStream>,
    security_keys: &[u8; 256],
    data_len:&AtomicU64
) -> io::Result<()> {
    let mut buf = [0u8; 1024 * 8];
    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        
        //统计流量
        data_len.fetch_add(n as u64, Ordering::Relaxed);
        
        //需要加密处理
        for b in &mut buf[..n] {
            *b = security_keys[*b as usize];
        }
        writer.write_all(&buf[..n]).await?;
    }

    //这里必须关闭客户端的输出流，否则对方无法感知到已经关闭连接了（写失败或者读失败没有必要调用shutdown()，即使调用大概率也是失败的，所以没有意义）
    writer.shutdown().await?;
    Ok(())
}

/// 明文传输
async fn traffic(
    mut reader: ReadHalf<TcpStream>,
    mut writer: WriteHalf<TcpStream>,
    data_len:&AtomicU64
) -> io::Result<()> {
    let mut buf = [0u8; 1024 * 8];
    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        
        //统计流量
        data_len.fetch_add(n as u64, Ordering::Relaxed);
        writer.write_all(&buf[..n]).await?;
    }

    //这里必须关闭客户端的输出流，否则对方无法感知到已经关闭连接了（写失败或者读失败没有必要调用shutdown()，即使调用大概率也是失败的，所以没有意义）
    writer.shutdown().await?;
    Ok(())
}