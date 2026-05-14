use crate::application::Argument;
use crate::npc_error::NpcError;
use crate::{application, tcp_pool};
use lib_np_common::{head_flag, time_util};
use std::io::ErrorKind;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::time::Duration;
use tokio::time::sleep;
use tokio::{io, select, try_join};

// 开启客户端
pub async fn open(args: Argument) {
    if *application::IS_OPENED.lock().unwrap() {
        //如果正在运行中
        return;
    }
    *application::IS_OPENED.lock().unwrap() = true;
    println!("-->NPC服务开启成功");
    *application::NPC_CONNECT_MSG.lock().unwrap() = "正在连接NPS服务器...".to_string();
    check_heart(args).await;

    //重置心跳时间,方便下次能快速连接
    application::LAST_HEART_TIME.store(0, Ordering::Relaxed);
    *application::IS_OPENED.lock().unwrap() = false;
}

pub async fn stop() {
    if !*application::IS_OPENED.lock().unwrap() {
        //如果没有运行中
        return;
    }

    //关闭npc服务
    shutdown_npc().await;
    while *application::IS_OPENED.lock().unwrap() {
        println!("-->正在停止服务...");
        application::APP_CLOSER.notify_waiters();
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}

// 检测心跳
async fn check_heart(args: Argument) {
    let app_closer = &application::APP_CLOSER;
    loop {
        select! {
            _ = app_closer.notified() => {
                // println!("收到关闭通知，准备关闭应用...");
                // 这里可以执行一些清理操作，例如关闭连接、释放资源等
                break;
            }

            _ = async{
                if time_util::current_millis() - application::LAST_HEART_TIME.load(Ordering::Relaxed)
                    > application::CHECK_HEART_TIME
                {

                    //长时间没有收到心跳，视为掉线
                    create_connection(args.clone()).await;
                }
                tokio::time::sleep(Duration::from_millis(application::CHECK_HEART_TIME)).await;
            } => {

            }
        }
    }
}

// 创建连接
async fn create_connection(args: Argument) {
    //关闭上次会话,application::NPC_CLOSER会重新进入等待状态
    shutdown_npc().await;
    *application::NPC_CONNECT_MSG.lock().unwrap() = "正在连接NPS服务器...".to_string();

    // 与服务端建立连接
    let nps_tcp = match TcpStream::connect(format!("{}:{}", args.host, args.tcp_port)).await {
        Ok(v) => v,
        Err(e) => {
            println!(
                "-->与主机连接失败:{}:{}  error:{:?}",
                args.host, args.tcp_port, e
            );
            *application::NPC_CONNECT_MSG.lock().unwrap() = format!(
                "与主机连接失败:{}:{}  错误:{:?}",
                args.host, args.tcp_port, e
            );
            return;
        }
    };

    tokio::spawn(async move {
        *application::IS_NPC_RUNNING.lock().unwrap() = true;
        spawn_start(args, nps_tcp).await;
        *application::IS_NPC_RUNNING.lock().unwrap() = false;

        println!("-->与服务端连接已经断开");
    });
}

async fn spawn_start(args: Argument, nps_tcp: TcpStream) {
    let closer = &application::NPC_CLOSER;
    select! {
        _ = closer.notified() => {
            // println!("收到关闭通知，准备关闭连接...");
            return;
        }
        result = start(args, nps_tcp) =>{
            match result {
                Err(NpcError::IoError(e)) if e.kind() == ErrorKind::UnexpectedEof =>{

                    //正常关闭
                    // println!("-->与服务端连接已经断开");
                }
                Err(e) =>{
                    println!("-->与主机通信发生了错误:{:?}", e);
                }
                _=>{}
            }
        }
    }
}

/**
 * 开始
 */
async fn start(args: Argument, mut nps_tcp: TcpStream) -> Result<(), NpcError> {
    //向服务器端发送客户端信息
    let header = format!("{}|{}", args.key, application::VERSION);
    nps_tcp
        .write_all(&[
            head_flag::CLIENT_TO_SERVER_MAIN_CONNECTION,
            header.len() as u8,
        ])
        .await?;
    nps_tcp.write_all(header.as_bytes()).await?;

    let flag = nps_tcp.read_u8().await?;
    match flag {
        head_flag::UNKNOW_KEY => {
            //未知的秘钥,直接返回,不再进行后续操作
            println!("-->未知的秘钥:{}", args.key);
            *application::NPC_CONNECT_MSG.lock().unwrap() = "未知的秘钥".to_string();
            return Ok(());
        }
        head_flag::DISABLED_KEY => {
            //未知的秘钥,直接返回,不再进行后续操作
            println!("-->该秘钥被禁用:{}", args.key);
            *application::NPC_CONNECT_MSG.lock().unwrap() = "该秘钥被禁用".to_string();
            return Ok(());
        }
        head_flag::CONNECT_SUCCESS => {
            *application::NPC_CONNECT_MSG.lock().unwrap() = "已连接到NPS服务器".to_string();
            //连接成功
        }
        _ => {
            return Err(NpcError::UnknowFlagError(flag));
        }
    }

    //获取客户端ID
    let client_id = nps_tcp.read_i64().await?;
    application::CLIENT_ID.store(client_id, Ordering::Relaxed);

    //客户端加解密秘钥
    let mut buf = [0u8; 256];
    nps_tcp.read_exact(&mut buf).await?;

    //更新秘钥
    *application::SECURITY_KEY.write().await = buf;

    println!("-->与服务端连接成功");
    let (reader, mut writer) = io::split(nps_tcp);
    let result = try_join!(heart(&mut writer), receive(args, reader));

    if let Err(e) = result {
        return Err(e);
    }
    Ok(())
}

/**
 * 从服务端收到数据
 */
async fn receive(args: Argument, mut reader: ReadHalf<TcpStream>) -> Result<(), NpcError> {
    loop {
        let flag = reader.read_u8().await?;
        //fmt.Printf("-->收到标记：%d : %c\n", flag, rune(flag))
        match flag {
            //服务器向客户端申请TCP连接池请求
            head_flag::REQUEST_TCP_POOL => {
                let count = reader.read_u8().await?;
                tcp_pool::create(args.clone(), count)
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
}

/**
 * 关闭服务
 */
async fn shutdown_npc() {
    while *application::IS_NPC_RUNNING.lock().unwrap() {
        println!("-->正在关闭NPC服务...");
        application::NPC_CLOSER.notify_waiters();
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
