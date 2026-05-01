use std::io::ErrorKind;
use crate::application;
use crate::tcp_bridge;
use crate::npc_error::NpcError;
use np_common::head_flag;
use std::sync::atomic::Ordering;
use std::sync::{Arc, LazyLock};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::select;

/// NPS服务端地址
static SERVER_ADDR: LazyLock<String> =
    LazyLock::new(|| format!("{}:{}", application::ARGS.host, application::ARGS.tcp_port));

// Create 创建TCP连接池
pub fn create(count: u8) {
    for _ in 0..count {
        tokio::spawn(async move{
            //桥接数+1
            application::POOL_COUNT.fetch_add(1, Ordering::Relaxed);

            // 由于[u8]已经实现了Copy,所以这里的security_key会被复制一份,由于数据量比较小,没有必要使用Arc或者Bytes,直接复制性能会更好
            spawn_start().await;

            //桥接数-1
            application::POOL_COUNT.fetch_sub(1, Ordering::Relaxed);
        });
    }
}

async fn spawn_start() {
    //npc服务端关闭通知
    let closer = &application::NPC_CLOSER;
    select! {
        _ = closer.notified() => {
            // println!("收到关闭通知，准备关闭连接池...");
            return;
        }
        result = start() => {
            match result {
                Err(NpcError::UnknowFlagError(flag)) => {
                    println!("-->未知的标记:{}", flag);
                }
                Err(NpcError::PoolIsFull) => {
                    println!("-->服务端连接池已满");
                }
                Err(NpcError::IoError(e)) if e.kind() == ErrorKind::UnexpectedEof => {

                    //正常关闭,不做任何处理
                    // println!("正常关闭,不做任何处理");
                }
                Err(e) => {
                    println!("连接池发生了错误:{:?}", e);
                }
                _ => {}
            }
        }
    }
}

async fn start() -> Result<(), NpcError> {
    
    //与目标端口建立连接
    let mut nps_tcp = TcpStream::connect(SERVER_ADDR.as_str()).await?;

    //发送头部标记
    nps_tcp.write_u8(head_flag::REQUEST_TCP_POOL).await?;

    //发送客户端id
    nps_tcp
        .write_i64(application::CLIENT_ID.load(Ordering::Relaxed))
        .await?;

    //等待分配工作
    wait_work(nps_tcp).await
}

/**
 * 等待分配工作
 */
async fn wait_work(mut nps_tcp: TcpStream) -> Result<(), NpcError> {
    // let flag = tokio::time::timeout(Duration::from_millis(10), nps_tcp.read_u8()).await;
    let flag = nps_tcp.read_u8().await?;
    match flag{
        head_flag::POOL_IS_FULL =>{
             Err(NpcError::PoolIsFull)
        }
        head_flag::CONNECT_TO_TARGET_SERVER =>{
            start_work(nps_tcp).await
        }
        _ =>{
             Err(NpcError::UnknowFlagError(flag))
        }
    }
}

async fn start_work(mut nps_tcp: TcpStream) -> Result<(), NpcError> {

    //加密类型及目标端口 格式:加密状态|端口  1|80   1|127.0.0.1:80
    //1:加密  0:不加密
    let info_len = nps_tcp.read_u8().await?;
    let mut header_data = vec![0u8; info_len as usize];
    nps_tcp.read_exact(&mut header_data).await?;
    let header = String::from_utf8_lossy(&header_data).into_owned();
    let headers: Vec<&str> = header.split("|").collect();
    if headers.len() != 2 {
        return Err(NpcError::InvalidHeader(header));
    }

    //加密状态  1:加密  0:不加密
    let is_encode_data = headers[0] == "1";

    //目标服务器信息
    let mut target_addr = headers[1].to_string();
    if !target_addr.contains(":") {
        //如果没有包含了ip地址,则默认是本地地址
        target_addr.insert_str(0, "127.0.0.1:");
    }
    tcp_bridge::ready(tcp_bridge::TCPBridgeParam {
        is_encode_data,
        nps_tcp,
        target_addr,
    });
    Ok(())
}
