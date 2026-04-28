use crate::application;
use crate::bridge::tcp_bridge;
use crate::npc_error::NpcError;
use np_common::head_flag;
use std::sync::atomic::Ordering;
use std::sync::{Arc, LazyLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::select;

/// NPS服务端地址
static SERVER_ADDR: LazyLock<String> =
    LazyLock::new(|| format!("{}:{}", application::ARGS.host, application::ARGS.tcp_port));

// Create 创建TCP连接池
pub fn create(count: u8) {
    for _ in 0..count {
        tokio::spawn(async {
            //桥接数+1
            application::POOL_COUNT.fetch_add(1, Ordering::Relaxed);

            spawn_start().await;

            //桥接数-1
            application::POOL_COUNT.fetch_sub(1, Ordering::Relaxed);
        });
    }
}

async fn spawn_start() {
    //npc服务端关闭通知
    let closer = application::NPC_CLOSER.load();
    select! {
        _ = closer.notified() => {
            println!("收到关闭通知，准备关闭连接池...");
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
    // removePool(mine)
}

// /**
//  * 发送客户端信息
//  */
// async fn send_client_info_to_server(
//     client_id: i64,
//     nps_tcp: &mut TcpStream,
// ) -> Result<(), NpcError> {
//     //将客户端id发送给NPS服务端
//     let header_data = header_util::make_header_data(
//         header_util::REQUEST_TCP_POOL,
//         client_id.to_string().as_str(),
//     );
//     nps_tcp.write_all(header_data.as_ref()).await?;
//     Ok(())
// }

/**
 * 等待分配工作
 */
async fn wait_work(mut nps_tcp: TcpStream) -> Result<(), NpcError> {
    let flag = nps_tcp.read_u8().await?;
    if flag == head_flag::POOL_IS_FULL {
        //服务端连接池已满
        return Err(NpcError::PoolIsFull);
    }
    if flag != head_flag::CONNECT_TO_TARGET_SERVER {
        return Err(NpcError::UnknowFlagError(flag));
    }

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
        // closer: Arc::new(Notify::new()),
    });
    Ok(())
}
