use std::sync::atomic::Ordering;
use std::sync::RwLock;
use bytes::Bytes;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;
use tokio::time::Duration;
use tokio::time::sleep;
use np_common::time_util;
use crate::{application, header_util};

use crate::npc_error::NpcError;
use crate::pool::tcp_pool;

// // 与服务端通信连接
// type NPCSession struct {
// 	npsTCP net.Conn
// }
//
// /**
//  * 开始
//  */
// func (mine *NPCSession) start() {
// 	mine.readServerInfoAndReceive()
//
// 	//关闭会话
// 	mine.npsTCP.Close()
// }
//
// // 从服务端读取基本信息
// func (mine *NPCSession) readServerInfoAndReceive() {
// 	header := constant.Key + "|" + constant.VERSION
//
// 	//发送标记信息
// 	if HeaderUtil.SendFlag(mine.npsTCP, HeaderUtil.CLIENT_TO_SERVER_MAIN_CONNECTION, header) != nil {
// 		return
// 	}
//
// 	//获取客户端ID
// 	if mine.readClientId() != nil {
// 		return
// 	}
//
// 	//客户端加解密秘钥
// 	if mine.readClientSecurityKey() != nil {
// 		return
// 	}
//
// 	//发送心跳数据
// 	go mine.heart()
//
// 	//接收数据
// 	mine.receive()
// }
//
// /**
//  * 获取客户端ID
//  */
// func (mine *NPCSession) readClientId() error {
//
// 	//第一个字节为标记
// 	flagData := make([]byte, 1)
// 	if _, err := io.ReadFull(mine.npsTCP, flagData); err != nil {
// 		return err
// 	}
// 	if flagData[0] != HeaderUtil.SERVER_TO_CLIENT_ID {
// 		mine.npsTCP.Close()
// 		return &extension.BusinessException{
// 			Message: "非法标记:$flag",
// 		}
// 	}
//
// 	//得到头部数据
// 	header, err := HeaderUtil.GetHeader(mine.npsTCP)
// 	if err != nil {
// 		return err
// 	}
//
// 	//得到客户端ID
// 	clientId, _ := strconv.ParseInt(header, 10, 64)
//
// 	//得到客户端ID
// 	constant.ClientId = int(clientId)
// 	return nil
// }
//
// 客户端加解密秘钥
async fn read_client_security_key(nps_tcp: &mut TcpStream) -> Result<(), NpcError>  {
	let mut buf = [0u8;256];
	nps_tcp.read_exact(&mut buf).await?;

	// clientSecurityKey := make([]byte, 256)
	// if _, err := io.ReadFull(mine.npsTCP, clientSecurityKey); err != nil {
	// 	return err
	// }

	//将数据复制到数组中
	copy(SecurityUtil.ClientSecurityKey[:], clientSecurityKey)
	return nil
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
			header_util::REQUEST_TCP_POOL => {
				let count = reader.read_u8().await?;
				// header, err := HeaderUtil.GetHeader(mine.npsTCP)
				// if err != nil {
				// 	return
				// }
				//
				// //创建数量
				// count, _ := strconv.ParseInt(header, 10, 64)
				//
				// //创建连接池
				// tcp_pool.Create(int(count))
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
			header_util::MAIN_HEART_BEAT => {


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

			_ =>{
				//未知的标记
				println!("-->未知的标记:{}",flag)
			}
		}
	}
}


// 定期心跳
async fn heart(sender: Sender<Bytes>, mut writer: WriteHalf<TcpStream>) -> Result<(), NpcError> {
	loop { //每个一段时间发送一次心跳包
		sleep(Duration::from_millis(application::HEART_TIME)).await;
		writer.write_u8(header_util::MAIN_HEART_BEAT).await?;
	}
	Ok(())
}

// /**
//  * 关闭服务
//  */
// func (mine *NPCSession) shutdown() {
// 	mine.npsTCP.Close()
// }
