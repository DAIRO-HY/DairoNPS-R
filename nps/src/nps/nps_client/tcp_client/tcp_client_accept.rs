use super::super::header_util;
use super::tcp_client_session_manager;
use crate::application;
use crate::dao::client_dao;
use crate::nps::nps_pool::tcp_pool;
use crate::util::time_util;
use sqlx::Error;
use std::net::SocketAddr;
use std::sync::atomic::Ordering;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use crate::nps_error::NpsError;

// 监听客户端连接
pub async fn accept() -> Result<(), NpsError> {
    let listener = TcpListener::bind("0.0.0.0:1781").await?;
    loop {
        select! {
        _ = application::SHUTDOWN_NOTIFY.notified() => {
                drop(listener);
                application::IS_NPS_SERVER_DROP.store(true, Ordering::Release);
                break;
        }
        acc = listener.accept() => {//等待客户端连接
                start(acc?)
            }
        }
    }
    println!("NPS服务端监听已关闭。");
    Ok(())
}

fn start((tcp_stream, addr): (TcpStream, SocketAddr)) {
    // println!("接收到客户端连接请求,端口:{}监听成功。", 1781);
    tokio::spawn(async move {
        match handle_accept(tcp_stream, addr).await {
            Err(NpsError::PoolIsFull) => {
                //Tcp连接池被填充满了，不做任何处理
                //println!("-->{}",NpsError::PoolIsFull)
            }
            Err(NpsError::IoError(e)) => {
                //@TODO: 应该写入日志
                println!("处理客户端连接发生错误:{:?}", e);
            }
            Ok(()) | _ => {}
        };
        // println!("客户端连接处理结束。");
    });
}

/**
 * 分配连接
 * @param socketClient 与客户端的连接
 */
async fn handle_accept(mut tcp_stream: TcpStream, addr: SocketAddr) -> Result<(), NpsError> {
    //读取连接的第一个数据,设置超时,避免恶意连接
    // tcp.SetReadDeadline(time.Now().Add(3 * time.Second))

    //读取第一个标记字节,通过该自己判断该连接类型
    let flag = tcp_stream.read_u8().await?;
    // println!("接收到客户端连接请求,标记:{}", flag);
    match flag {
        //标记该连接为:服务器端往客户端发送指令的连接
        header_util::CLIENT_TO_SERVER_MAIN_CONNECTION => validate_session(tcp_stream, addr).await?,

        //创建客户端Socket连接池
        header_util::REQUEST_TCP_POOL => tcp_pool::add(tcp_stream).await?,

        _ => {
            println!("未知标记:{}", flag);
        }
    }
    Ok(())
}

// 验证客户端回话
async fn validate_session(mut tcp_stream: TcpStream, addr: SocketAddr) -> Result<(), NpsError> {
    //得到头部数据
    let header = header_util::get_header(&mut tcp_stream).await?;
    let headers: Vec<&str> = header.split("|").collect();

    //得到客户端key
    let key = headers[0];
    let client = match client_dao::select_by_key(&db::get(), key).await {
        Ok(v) => v,
        Err(Error::RowNotFound) => {
            println!("客户端：{}获取失败", key);
            tcp_stream.shutdown().await?;
            return Ok(());
        }
        Err(e) => {
            println!("客户端：{}获取失败：{}", key, e);
            tcp_stream.shutdown().await?;
            return Ok(());
        }
    };
    if !client.is_enabled {
        // println!("客户端：{}已被停止服务,IP:%s", key);
        tcp_stream.shutdown().await?;
        return Ok(());
    }
    let client_id = client.id;

    //得到客户端版本号
    let client_version = headers[1].to_string();
    let _ = client_dao::set_connection_info(
        &db::get(),
        client_id,
        addr.ip().to_string(),
        client_version,
        time_util::current_millis() as i64,
    )
    .await;
    tcp_client_session_manager::hold_on_client(client, tcp_stream).await
}
