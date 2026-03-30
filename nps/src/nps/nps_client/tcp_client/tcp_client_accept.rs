use super::super::header_util;
use super::tcp_client_session_manager;
use crate::dao::client_dao;
use crate::nps::nps_pool::tcp_pool_manager;
use tokio::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use crate::application;
use std::sync::atomic::Ordering;

// 监听客户端连接
pub async fn accept() -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:1781").await?;
    loop {
        select! {
        _ = application::SHUTDOWN_NOTIFY.notified() => {
            drop(listener);
            application::IS_NPS_SERVER_DROP.store(true, Ordering::Release);
            break;
        }
        acc = listener.accept() => {//等待客户端连接
                       let (tcp_stream, _) = acc?;
                       println!("接收到客户端连接请求,端口:{}监听成功。", 1781);
                       tokio::spawn(async {
                           if handle_accept(tcp_stream).await.is_err() {
                               println!("处理客户端连接发生错误。");
                           }
                           println!("客户端连接处理结束。");
                       });
                   }
                   }
    }
    println!("NPS服务端监听已关闭。");
    Ok(())
}

/**
 * 分配连接
 * @param socketClient 与客户端的连接
 */
async fn handle_accept(mut tcp_stream: TcpStream) -> io::Result<()> {
    //读取连接的第一个数据,设置超时,避免恶意连接
    // tcp.SetReadDeadline(time.Now().Add(3 * time.Second))

    //读取第一个标记字节,通过该自己判断该连接类型
    let mut flag_data = [0u8; 1];
    tcp_stream.read_exact(&mut flag_data).await?;
    let flag = flag_data[0];
    println!("接收到客户端连接请求,标记:{}", flag);
    match flag {
        //标记该连接为:服务器端往客户端发送指令的连接
        header_util::CLIENT_TO_SERVER_MAIN_CONNECTION => validate_session(tcp_stream).await?,

        //创建客户端Socket连接池
        header_util::REQUEST_TCP_POOL => tcp_pool_manager::add(tcp_stream).await?,

        _ => {}
    }
    Ok(())
}

// 验证客户端回话
async fn validate_session(mut tcp_stream: TcpStream) -> io::Result<()> {
    //得到头部数据
    let header = header_util::get_header(&mut tcp_stream).await?;
    let headers: Vec<&str> = header.split("|").collect();

    //得到客户端key
    let key = headers[0];
    let conn = crate::util::db_util::new_connection();
    let client = client_dao::select_by_key(&conn, key);
    drop(conn);
    if let Err(e) = client {
        if e != rusqlite::Error::QueryReturnedNoRows{
            println!("客户端：{}获取失败:{}", key,e);
        }
        tcp_stream.shutdown().await?;
        return Ok(());
    }
    let client = client.unwrap();
    if client.enable_state == 0 {
        // println!("客户端：{}已被停止服务,IP:%s", key);
        tcp_stream.shutdown().await?;
        return Ok(());
    }

    // //设置客户端登录信息-------------------------------------------------------------------------------START
    // remoteAddr := tcp.RemoteAddr().String()
    //
    // //客户端ip
    // ip := strings.Split(remoteAddr, ":")[0]
    //
    // //从头部信息中得到客户端版本号
    // version := headers[1]
    // loginClientDto := dto.ClientDto{
    //     Id:      client.Id,
    //     Ip:      ip,
    //     Version: version,
    // }
    // ClientDao.SetClientInfo(loginClientDto)
    // //设置客户端登录信息-------------------------------------------------------------------------------END
    //
    tcp_client_session_manager::hold_on_client(client, tcp_stream).await?;
    Ok(())
}
