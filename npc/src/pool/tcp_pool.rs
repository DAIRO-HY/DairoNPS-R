use crate::bridge::tcp_bridge;
use crate::npc_error::NpcError;
use crate::{application, header_util};
use std::fmt::format;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, LazyLock};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Notify;
// // TCPPool 等待分配工作的Socket
// type TCPPool struct {
// 	npsTCP net.Conn
// }

/// NPS服务端地址
static SERVER_ADDR: LazyLock<String> =
    LazyLock::new(|| format!("{}:{}", application::ARGS.host, application::ARGS.tcp_port));

// Create 创建TCP连接池
pub fn create(count: u8) {
    for i in 1..count {
        tokio::spawn(async move {
            
            //与目标端口建立连接
            let Ok(nps_tcp) = TcpStream::connect(SERVER_ADDR.as_str()) else {
                return;
            };
            let client_id = 1;
            if let Err(e) = start(client_id, nps_tcp).await {
                println!("连接池发生了错误:{:?}", e);
            }
        });
    }

    //与目标端口建立连接
    // tcp, err := net.Dial("tcp", constant.Host+":"+constant.TcpPort)
    // if err != nil {
    // return
    // }
    // pool := &TCPPool{
    // npsTCP: tcp,
    // }
    // lock.Lock()
    // mTcpPoolList[pool] = true
    // lock.Unlock()
    // go pool.start()
    // }
}

// /**
//  * 开始等待分配工作
//  */
//  fn ready(client_id:i64, nps_tcp: TcpStream) {
// 	tokio::spawn(async move {
// 		if let Err(e) = start(client_id, nps_tcp).await {
// 			println!("连接池发生了错误:{:?}", e);
// 		}
// 	});
// }

async fn start(client_id: i64, mut nps_tcp: TcpStream) -> Result<(), NpcError> {
    //发送客户端信息
    send_client_info_to_server(client_id, &mut nps_tcp).await?;

    //等待分配工作
    wait_work(nps_tcp).await
    // removePool(mine)
}

/**
 * 发送客户端信息
 */
async fn send_client_info_to_server(
    client_id: i64,
    nps_tcp: &mut TcpStream,
) -> Result<(), NpcError> {
    //将客户端id发送给NPS服务端
    let header_data = header_util::make_header_data(
        header_util::REQUEST_TCP_POOL,
        client_id.to_string().as_str(),
    );
    nps_tcp.write_all(header_data.as_ref()).await?;
    Ok(())
}

/**
 * 等待分配工作
 */
async fn wait_work(mut nps_tcp: TcpStream) -> Result<(), NpcError> {
    //加密类型及目标端口 格式:加密状态|端口  1|80   1|127.0.0.1:80
    //1:加密  0:不加密
    let header = header_util::get_header(&mut nps_tcp).await?;
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
        closer: Arc::new(Notify::new()),
        bridge_count: Arc::new(AtomicUsize::new(0)),
    });
    Ok(())
}
