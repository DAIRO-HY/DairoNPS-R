use crate::dao::channel_dao;
use crate::dao::channel_dao::Channel;
use crate::nps::nps_bridge::tcp_bridge;
use crate::nps::nps_pool::tcp_pool;
use crate::nps_error::NpsError;
use crate::{application, nps};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tokio::{net::TcpListener, select, sync::Notify};
use lib_np_common::data_io_len::AtomicDataIOLen;

// 开始客户端的所有监听
pub async fn ready_client(client_id: i64) -> Result<(), NpsError> {
    //开启NPS客户端ID下所有的隧道
    let active_list = channel_dao::select_active_by_client_id(&mut lib_db::get_context(), client_id).await?;
    for it in active_list {
        if it.mode == 1 {
            //只监听TCP隧道
            ready_channel(it).await?;
        }
    }
    Ok(())
}

// 开始监听某个隧道
pub async fn ready_channel(channel: Channel) -> Result<(), NpsError> {
    tokio::spawn(async move {
        let channel_id = channel.id;
        spawn_start(channel).await;
        
        //监听结束后,移除隧道监听信息
        nps::CHANNEL_LIVE_MAP.lock().await.remove(&channel_id);
    });
    Ok(())
}

async fn spawn_start(channel: Channel) {
    
    //关闭隧道正在通信的连接
    shutdown_by_channel(channel.id).await;
    let client_id = channel.client_id;
    let channel_id = channel.id;
    let data_len = AtomicDataIOLen::from(channel.in_len, channel.out_len);
    let channel_closer = Arc::new(Notify::new());
    let bridge_count = Arc::new(AtomicUsize::new(0));

    //保存关闭通知器
    nps::CHANNEL_LIVE_MAP.lock().await.insert(
        channel_id,
        nps::ChannelLive {
            client_id,
            data_len: data_len.clone(),
            closer: channel_closer.clone(),
            bridge_count: bridge_count.clone(),
        },
    );

    let _channel_closer = channel_closer.clone();
    select! {
        // 接收到关闭通知：退出 accept 循环
        _ = _channel_closer.notified() => {
            return;
        }
        // 接收到全局关闭通知：退出 accept 循环
        _ = application::SHUTDOWN_NOTIFY.notified() => {
            return;
        }
        result = start(channel,channel_closer,data_len,bridge_count) => {
            if let Err(e) = result {
                println!("监听隧道发生了错误:{:?}", e);
            }
        }
    }
}

/**
 * 等待代理客户端连接
 */
async fn start(
    channel: Channel,
    channel_closer: Arc<Notify>,
    data_len: AtomicDataIOLen,
    bridge_count: Arc<AtomicUsize>,
) -> Result<(), NpsError> {
    let tcp_listener = match TcpListener::bind(format!("0.0.0.0:{}", channel.server_port)).await {
        Ok(v) => v,
        Err(e) => {
            channel_dao::set_error(&mut lib_db::get_context(), channel.id, format!("监听端口失败:{:?}", e)).await?;
            // return Err(NpsError::IoError(e));
            return Ok(());
        }
    };

    //清除错误消息
    channel_dao::clear_error(&mut lib_db::get_context(), channel.id).await?;
    loop {
        let (mut proxy_tcp, addr) = tcp_listener.accept().await?;

        //从连接池里取出一个连接
        let Some(client_tcp) = tcp_pool::get_and_add_pool(channel.client_id).await else {
            continue;
        };
        tcp_bridge::ready(
            tcp_bridge::TcpBridgeParam {
                ip: addr.ip().to_string(),
                channel_id: channel.id,
                is_stats_traffic: channel.is_stats_traffic,
                target_port: channel.target_port.clone(),
                security_state: channel.security_state,
                proxy_tcp,
                client_tcp,
                data_len: data_len.clone(),
            },
            bridge_count.clone(),
            channel_closer.clone(),
        )
        .await;
    }
}

// 关闭监听
// - channelId 隧道id
pub async fn shutdown_by_channel(channel_id: i64) {
    loop {
        //等待隧道代理监听停止,否则可能导致下次监听同一端口失败
        if let Some(channel_live) = nps::CHANNEL_LIVE_MAP.lock().await.get(&channel_id) {
            let _ = channel_live.closer.notify_waiters();
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        } else {
            return;
        }
    }
}
