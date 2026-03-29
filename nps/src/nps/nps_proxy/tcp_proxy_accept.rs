use crate::dao::channel_dao::Channel;
use crate::model::data_total::DataTotal;
use crate::nps;
use crate::nps::nps_bridge::tcp_bridge::TCPBridgeInfo;
use crate::nps::nps_bridge::tcp_bridge_manager;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use tokio::net::TcpListener;
use tokio::select;
use tokio::sync::Notify;

/**
 * TCP隧道代理
 */
pub struct TCPProxyAccept {
    pub channel: Channel, //隧道信息
    pub tcp_listener: TcpListener,
    pub notify: Arc<Notify>,
    pub data_total: DataTotal,
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
    pub async fn accept(self) -> tokio::io::Result<()> {
        //初始化隧道对应的桥接信息
        let bridge_info_map = Arc::new(DashMap::new());
        nps::BRIDGE_INFO
            .lock()
            .await
            .insert(self.channel.id, bridge_info_map.clone());
        loop {
            select! {
                // 接收到关闭通知：退出 accept 循环
                _ = self.notify.notified() => {
                    println!("-->接收到关闭通知：退出 accept 循环");
                    break;
                }

                accept_res = self.tcp_listener.accept() => {
                    let (proxy_tcp,_) = accept_res?;
                    println!("-->监听到代理隧道: {} 服务端口: {} 目标端口: {}", self.channel.id, self.channel.server_port, self.channel.target_port);
                    tcp_bridge_manager::make_bridge(
                        bridge_info_map.clone(),
                        &(self.channel),
                        proxy_tcp,
                        self.data_total.clone(),
                        self.notify.clone(),
                    ).await;
                }
            }
        }
        Ok(())
    }
}
