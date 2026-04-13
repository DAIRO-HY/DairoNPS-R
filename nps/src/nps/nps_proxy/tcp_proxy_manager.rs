use crate::dao::channel_dao;
use crate::dao::channel_dao::Channel;
use crate::model::data_io_len::AtomicDataIOLen;
use crate::nps;
use crate::nps::nps_proxy::tcp_proxy_accept::TCPProxyAccept;
use crate::nps_error::NpsError;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use tokio::{net::TcpListener, sync::Notify};

// 开始客户端的所有监听
pub async fn accept_client(client_id: i64) -> Result<(), NpsError> {
    //开启NPS客户端ID下所有的隧道
    let active_list = channel_dao::select_active_by_client_id(&db::get(), client_id).await?;
    for it in active_list {
        if it.mode == 1 {
            //只监听TCP隧道
            accept_channel(it).await?;
        }
    }
    Ok(())
}

// 开始监听某个隧道
pub async fn accept_channel(channel: Channel) -> Result<(), NpsError> {
    //关闭隧道正在通信的连接
    shutdown_by_channel(channel.id).await;
    let tcp_listener = match TcpListener::bind(format!("0.0.0.0:{}", channel.server_port)).await {
        Ok(v) => v,
        Err(e) => {
            channel_dao::set_error(&db::get(), channel.id, format!("监听端口失败:{:?}", e)).await?;
            return Ok(());
        }
    };

    //清除错误消息
    channel_dao::clear_error(&db::get(), channel.id).await?;

    let client_id = channel.client_id;
    let channel_id = channel.id;
    let data_len = AtomicDataIOLen::from(channel.in_len, channel.out_len);
    let closer = Arc::new(Notify::new());
    let bridger = Arc::new(DashMap::new());
    let bridge_count = Arc::new(AtomicUsize::new(0));

    let proxy_tcp_accept = TCPProxyAccept {
        channel,
        tcp_listener,
        closer: closer.clone(),
        data_len: data_len.clone(),
        bridger: bridger.clone(),
        bridge_count: bridge_count.clone(),
    };

    //保存关闭通知器
    nps::CHANNEL_LIVE_MAP.lock().await.insert(
        channel_id,
        nps::ChannelLive {
            client_id,
            data_len,
            closer,
            bridger,
            bridge_count,
        },
    );
    tokio::spawn(async move {
        if let Err(e) = proxy_tcp_accept.accept().await {
            println!("监听隧道发生了错误:{:?}", e);
        }

        //接受到accept结束的通知,说明监听已经停止,可以安全地删除关闭通知器
        nps::CHANNEL_LIVE_MAP.lock().await.remove(&channel_id);
    });
    Ok(())
}

// 关闭监听
// - channelId 隧道id
pub async fn shutdown_by_channel(channel_id: i64) {
    // proxyAcceptLock.Lock()
    // proxyTCPAccept := proxyAcceptMap[channelId]
    // if proxyTCPAccept != nil {
    // 	shutdown(proxyTCPAccept)
    // }
    // proxyAcceptLock.Unlock()

    loop {
        //等待隧道代理监听停止,否则可能导致下次监听同一端口失败
        if let Some(channel_live) = nps::CHANNEL_LIVE_MAP.lock().await.get(&channel_id) {
            let _ = channel_live.closer.notify_waiters();
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        } else {
            return;
        }
    }
    // ss.map(|notify| {
    // 	notify.notify_one();
    // });

    //关闭隧道所有正在通信的连接
    // tcp_bridge.ShutdownByChannel(channelId)
}

// // 关闭某个客户端下所有的隧道
// func ShutdownByClient(clientId int) {

// 	//关闭客户端所有隧道
// 	channelIdList := ChannelDao.SelectIdByClientId(clientId)
// 	for _, it := range channelIdList {
// 		ShutdownByChannel(it)
// 	}

// 	//关闭客户端所有正在通信的连接
// 	tcp_bridge.ShutdownByClient(clientId)
// }

// // 停止监听端口
// func shutdown(proxyTCPAccept *TCPProxyAccept) {
// 	proxyTCPAccept.listen.Close()
// 	channelId := proxyTCPAccept.Channel.Id
// 	if proxyAcceptMap[channelId] != nil {
// 		delete(proxyAcceptMap, channelId)
// 	}
// }
