use crate::model::data_io_len::AtomicDataIOLen;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio::sync::Notify;
use crate::application;
use crate::dao::forward_dao::Forward;
use crate::forward::tcp_bridge::TCPBridgeInfo;
use crate::forward::tcp_bridge_manager;
use crate::nps_error::NpsError;

/**
 * TCP端口转发代理
 */
pub struct TCPForwardAccept {
	pub forward: Forward, //端口转发信息
	pub tcp_listener: TcpListener,
	pub closer: Arc<Notify>,
	pub data_len: AtomicDataIOLen,
	pub bridger:Arc<DashMap<u64, TCPBridgeInfo>>,
}

impl TCPForwardAccept {

	/**
	 * 等待代理客户端连接
	 */
	pub async fn accept(self) -> Result<(), NpsError> {
		loop {
			select! {
                // 接收到关闭通知：退出 accept 循环
                _ = self.closer.notified() => {
                    println!("-->接收到关闭通知：退出 accept 循环");
                    break;
                }
                // 接收到全局关闭通知：退出 accept 循环
                _ = application::SHUTDOWN_NOTIFY.notified() => {
                    break;
                }

                accept_res = self.tcp_listener.accept() => {
                    let (proxy_tcp,_) = accept_res?;
                    // println!("-->监听到端口转发代理: {} 服务端口: {} 目标端口: {}", self.channel.id, self.channel.server_port, self.channel.target_port);

                    
                    tcp_bridge_manager::make_bridge(
                        self.bridger.clone(),
                        &(self.forward),
                        proxy_tcp,
                        self.data_len.clone(),
                        self.closer.clone(),
                    ).await;
                }
            }
		}
		Ok(())
	}
}
