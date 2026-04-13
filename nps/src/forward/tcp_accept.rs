use crate::application;
use crate::dao::forward_dao::Forward;
use crate::forward::tcp_bridge::TCPBridgeInfo;
use crate::forward::tcp_bridge_manager;
use crate::model::data_io_len::AtomicDataIOLen;
use crate::nps_error::NpsError;
use dashmap::DashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::Notify;

/**
 * 等待代理客户端连接
 */
pub async fn accept(
    forward: Forward, //端口转发信息
    tcp_listener: TcpListener,
    closer: Arc<Notify>,
    data_len: AtomicDataIOLen,
    bridger: Arc<DashMap<u64, TCPBridgeInfo>>,
    bridge_count: Arc<AtomicUsize>,
) -> Result<(), NpsError> {
    loop {
        select! {
            // 接收到关闭通知：退出 accept 循环
            _ = closer.notified() => {
                println!("-->接收到关闭通知：退出 accept 循环");
                break;
            }
            // 接收到全局关闭通知：退出 accept 循环
            _ = application::SHUTDOWN_NOTIFY.notified() => {
                break;
            }

            accept_res = tcp_listener.accept() => {
                let (proxy_tcp,_) = accept_res?;
                tcp_bridge_manager::make_bridge(
                    bridger.clone(),
                    &(forward),
                    proxy_tcp,
                    data_len.clone(),
                    closer.clone(),
                    bridge_count.clone()
                ).await;
            }
        }
    }
    Ok(())
}
