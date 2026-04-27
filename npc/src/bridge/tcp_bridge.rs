use crate::application;
use crate::npc_error::NpcError;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Notify;
use tokio::{io, select};

pub struct TCPBridgeParam {
    //数据是否加密
    pub is_encode_data: bool,

    //与NPS服务端的TCP
    pub nps_tcp: TcpStream,

    //目标服务器的地址
    pub target_addr: String,
    // pub closer: Arc<Notify>,
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
        if let Err(e) = start(param).await {
            println!("桥接通信接发生了错误:{:?}", e);
        }
    });
}

// // 读取缓存大小(最好和服务器端保持一致)
// const READ_CACHE_SIZE = 32 * 1024;

// TCP桥接通信开始
pub async fn start(mut param: TCPBridgeParam) -> Result<(), NpcError> {
	//桥接数+1
    application::BRIDGE_COUNT.fetch_add(1, Ordering::Relaxed);

    // 连接到服务器
    // //与目标端口建立连接
    // tcp, err := net.Dial("tcp", targetAddr)
    // if err != nil {
    // 	mine.NpsTCP.Close()
    // 	return
    // }
    // mine.TargetTCP = tcp
    // go mine.receiveByNpsSendToTarget()
    // mine.receiveByTargetSendToNps()



	// 建立连接
	let target_tcp = match TcpStream::connect(param.target_addr).await {
		Ok(v) => v,
		Err(e) => {
			//与目标服务器连接失败时，直接关闭
			let _ = param.nps_tcp.shutdown().await;

			//桥接数-1
            application::BRIDGE_COUNT.fetch_sub(1, Ordering::Relaxed);
			return Err(NpcError::IoError(e));
		}
	};
    let result = copy(param.nps_tcp, target_tcp).await;

	//桥接数-1
    application::BRIDGE_COUNT.fetch_sub(1, Ordering::Relaxed);
	result
}

async fn copy(
	mut nps_tcp: TcpStream,
	mut target_tcp: TcpStream,
	// closer: Arc<Notify>,
) -> Result<(), NpcError> {
    // select! {
    //     _ = closer.notified() => {
    //         nps_tcp.shutdown().await?;
    //         target_tcp.shutdown().await?;
    //         return Ok(());
    //     }
    //     rs = io::copy_bidirectional(&mut target_tcp, &mut nps_tcp) =>{
    //         return match rs{
    //             Ok(_)=>{
    //                 Ok(())
    //             },
    //             Err(e) =>{
    //                 Err(NpcError::IoError(e))
    //             }
    //         }
    //     }
    // }
    if let Err(e) = io::copy_bidirectional(&mut target_tcp, &mut nps_tcp).await{
        return Err(NpcError::IoError(e));
    }
    Ok(())
}

//
// // 从内网穿透服务器接收数据,发送到目标端口
// func (mine *TCPBridge) receiveByNpsSendToTarget() {
// 	data := make([]uint8, READ_CACHE_SIZE)
// 	for {
// 		n, readErr := mine.NpsTCP.Read(data)
// 		if n > 0 {
//
// 			//数据解密
// 			if mine.isEncodeData {
// 				SecurityUtil.Mapping(data, n)
// 			}
//
// 			//从代理端读取到的数据立即发送目标端
// 			if err := WriterUtil.WriteFull(mine.TargetTCP, data[:n]); err != nil {
// 				break
// 			}
// 		}
// 		if readErr != nil {
// 			break
// 		}
// 	}
//
// 	//关闭代理端的读操作
// 	mine.NpsTCP.(*net.TCPConn).CloseRead()
//
// 	//关闭目标端的写操作
// 	mine.TargetTCP.(*net.TCPConn).CloseWrite()
//
// 	//标记代理端读操作已经关闭
// 	mine.isNpcReadClosed = true
// 	mine.recycle()
// }
//
// // 从目标端口接收到数据,发送到内网穿透服务器
// func (mine *TCPBridge) receiveByTargetSendToNps() {
// 	data := make([]uint8, READ_CACHE_SIZE)
// 	for {
// 		n, readErr := mine.TargetTCP.Read(data)
// 		if n > 0 {
//
// 			//数据解密
// 			if mine.isEncodeData {
// 				SecurityUtil.Mapping(data, n)
// 			}
//
// 			//往NPS服务器发送数据
// 			if err := WriterUtil.WriteFull(mine.NpsTCP, data[:n]); err != nil {
// 				break
// 			}
// 		}
// 		if readErr != nil {
// 			break
// 		}
// 	}
//
// 	//关闭目标端的读操作
// 	mine.TargetTCP.(*net.TCPConn).CloseRead()
//
// 	//关闭NPS服务端的写操作
// 	mine.NpsTCP.(*net.TCPConn).CloseWrite()
//
// 	//标记目标端读操作已经关闭
// 	mine.isTargetReadClosed = true
// 	mine.recycle()
// }

// // 回收连接
// func (mine *TCPBridge) recycle() {
// 	if mine.isNpcReadClosed && mine.isTargetReadClosed {
// 		mine.NpsTCP.Close()
// 		mine.TargetTCP.Close()
// 		removeBridge(mine)
// 	}
// }
//
// // 关闭链接
// func (mine *TCPBridge) shutdown() {
// 	mine.NpsTCP.Close()
// 	mine.TargetTCP.Close()
// }
