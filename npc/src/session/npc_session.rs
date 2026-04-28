use crate::application;
use bytes::Bytes;
use np_common::{head_flag, time_util};
use std::sync::atomic::Ordering;
use std::sync::{Arc, RwLock};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Notify;
use tokio::sync::mpsc::Sender;
use tokio::time::Duration;
use tokio::time::sleep;
use tokio::{io, select, try_join};

use crate::npc_error::NpcError;
use crate::pool::tcp_pool;

// // 与服务端通信连接
// type NPCSession struct {
// 	npsTCP net.Conn
// }
//

// 开启客户端
pub async fn open() {
    if *application::IS_RUNNING.lock().await {
        //如果正在运行中
        return;
    }
    *application::IS_RUNNING.lock().await = true;
    application::APP_CLOSER.store(Arc::new(Notify::new()));
    println!("NPC服务开启成功");
    check_heart().await;
    *application::IS_RUNNING.lock().await = false;
}

pub async fn close() {
    if !*application::IS_RUNNING.lock().await {
        //如果没有运行中
        return;
    }

    //关闭npc服务
    shutdown_npc().await;
    while *application::IS_RUNNING.lock().await {
        println!("正在关闭服务...");
        application::APP_CLOSER.load().notify_waiters();
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    println!("正在关闭NPC服务...");
}

// 检测心跳
async fn check_heart() {
    let app_closer = application::APP_CLOSER.load();
    loop {
        select! {
            _ = app_closer.notified() => {
                println!("收到关闭通知，准备关闭应用...");
                // 这里可以执行一些清理操作，例如关闭连接、释放资源等
                break;
            }

            _ = async{
                if time_util::current_millis() - application::LAST_HEART_TIME.load(Ordering::Relaxed)
                    > application::CHECK_HEART_TIME
                {
                    //长时间没有收到心跳，视为掉线

                    //关闭上次会话
                    shutdown_npc().await;
                    create_connection().await;
                }
                tokio::time::sleep(Duration::from_millis(application::CHECK_HEART_TIME)).await;
            } => {

            }
        }
    }
}

// 创建连接
async fn create_connection() {
    // 与服务端建立连接
    let nps_tcp = match TcpStream::connect(format!(
        "{}:{}",
        application::ARGS.host,
        application::ARGS.tcp_port
    ))
    .await
    {
        Ok(v) => v,
        Err(e) => {
            println!(
                "-->与主机连接失败:{}:{}  error:{:?}",
                application::ARGS.host,
                application::ARGS.tcp_port,
                e
            );
            return;
        }
    };
    println!("-->与主机连接成功");
    tokio::spawn(async move {
        *application::IS_NPC_RUNNING.lock().await = true;
        spawn_start(nps_tcp).await;
        *application::IS_NPC_RUNNING.lock().await = false;
    });
}

async fn spawn_start(nps_tcp: TcpStream) {
    let closer = Arc::new(Notify::new());

    // 将关闭通知器存储到全局变量中，以便其他部分可以发送关闭通知
    application::NPC_CLOSER.store(closer.clone());
    select! {
        _ = closer.notified() => {
            println!("收到关闭通知，准备关闭连接...");
            return;
        }
        result = start(nps_tcp) =>{
            if let Err(e) = result {
                println!("-->与主机通信发生了错误:{:?}", e);
            }
        }
    }
}

/**
 * 开始
 */
async fn start(mut nps_tcp: TcpStream) -> Result<(), NpcError> {
    //向服务器端发送客户端信息
    let header = format!("{}|{}", application::ARGS.key, application::VERSION);
    nps_tcp
        .write_all(&[
            head_flag::CLIENT_TO_SERVER_MAIN_CONNECTION,
            header.len() as u8,
        ])
        .await?;
    nps_tcp.write_all(header.as_bytes()).await?;

    //获取客户端ID
    let client_id = nps_tcp.read_i64().await?;
    application::CLIENT_ID.store(client_id, Ordering::Relaxed);

    //客户端加解密秘钥
    let mut buf = [0u8; 256];
    nps_tcp.read_exact(&mut buf).await?;
    application::SECURITY_KEY.store(Arc::from(buf));

    let (reader, mut writer) = io::split(nps_tcp);

    let result = try_join!(heart(&mut writer), receive(reader));
    if let Err(e) = result {
        return Err(e);
    }
    Ok(())
}

// 客户端加解密秘钥
async fn read_client_security_key(mut nps_tcp: TcpStream) -> Result<(), NpcError> {
    let mut buf = [0u8; 256];
    nps_tcp.read_exact(&mut buf).await?;
    application::SECURITY_KEY.store(Arc::from(buf));
    Ok(())
}

/**
 * 从服务端收到数据
 */
async fn receive(mut reader: ReadHalf<TcpStream>) -> Result<(), NpcError> {
    loop {
        let flag = reader.read_u8().await?;
        //fmt.Printf("-->收到标记：%d : %c\n", flag, rune(flag))
        match flag {
            //服务器向客户端申请TCP连接池请求
            head_flag::REQUEST_TCP_POOL => {
                let count = reader.read_u8().await?;
                tcp_pool::create(count)
            }

            // //服务器向客户端申请UDP连接池请求
            // case HeaderUtil.REQUEST_UDP_POOL:
            // 	header, err := HeaderUtil.GetHeader(mine.npsTCP)
            // 	if err != nil {
            // 		return
            // 	}
            //
            // 	//创建数量
            // 	count, _ := strconv.ParseInt(header, 10, 64)
            //
            // 	//创建连接池
            // 	udp_pool.Create(int(count))
            //
            //服务器端回复了心跳
            head_flag::MAIN_HEART_BEAT => {
                //println("-->收到服务器心跳数据:${System.currentTimeMillis()}")
                //fmt.Printf("当前UDP连接池:%d UDP桥接数:%d \n", udp_pool.Count(), udp_bridge.Count())
                // lastHeartTime = time.Now().UnixMilli()
                application::LAST_HEART_TIME.store(time_util::current_millis(), Ordering::Relaxed)
            }

            ////服务器向客户端同步当前处于激活状态的UDP连接池端口
            //case HeaderUtil.SYNC_ACTIVE_POOL_UDP_PORT : {
            //    val ports = HeaderUtil.getHeader(this.npcTCP) ?: continue
            //    UDPPoolManager.syncServerActivePort(ports)
            //}
            //
            // //向客户端同步当前保留的UDP连接端口
            // case HeaderUtil.SYNC_ACTIVE_BRIDGE_UDP_PORT:
            // 	ports, err := HeaderUtil.GetHeader(mine.npsTCP)
            // 	if err != nil {
            // 		return
            // 	}
            // 	fmt.Println(ports)
            // 	//UDPBriageManager.syncServerActivePort(ports)
            //
            // }
            _ => {
                //未知的标记
                println!("-->未知的标记:{}", flag)
            }
        }
    }
}

// 定期心跳
async fn heart(writer: &mut WriteHalf<TcpStream>) -> Result<(), NpcError> {
    loop {
        //每个一段时间发送一次心跳包

        sleep(Duration::from_millis(application::HEART_TIME)).await;
        writer.write_u8(head_flag::MAIN_HEART_BEAT).await?
    }
    Ok(())
}

/**
 * 关闭服务
 */
async fn shutdown_npc() {
    while *application::IS_NPC_RUNNING.lock().await {
        println!("正在关闭NPC服务...");
        application::NPC_CLOSER.load().notify_waiters();
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
