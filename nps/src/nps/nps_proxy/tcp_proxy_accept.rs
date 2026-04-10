use crate::dao::channel_dao::Channel;
use crate::model::data_io_len::AtomicDataIOLen;
use crate::nps::nps_bridge::tcp_bridge_manager;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::Notify;
use crate::application;
use crate::nps::nps_bridge::tcp_bridge::TCPBridgeInfo;
use crate::nps::nps_error::NpsError;

/**
 * TCP隧道代理
 */
pub struct TCPProxyAccept {
    pub channel: Channel, //隧道信息
    pub tcp_listener: TcpListener,
    pub closer: Arc<Notify>,
    pub data_len: AtomicDataIOLen,
    pub bridger:Arc<DashMap<u64, TCPBridgeInfo>>,
}

impl TCPProxyAccept {
    /**
     * 访问控制的IP地址
     */
    //private val aclIpSet = ChannelAclDao.selectByChannelId(channel.id!!).map {
    //    it.ip!!
    //}.toSet()

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
                    // println!("-->监听到代理隧道: {} 服务端口: {} 目标端口: {}", self.channel.id, self.channel.server_port, self.channel.target_port);
                    tcp_bridge_manager::make_bridge(
                        self.bridger.clone(),
                        &(self.channel),
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
