use crate::dao::forward_dao;
use crate::dao::forward_dao::Forward;
use crate::forward::tcp_bridge;
use crate::nps_error::NpsError;
use crate::{application, forward};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tokio::{net::TcpListener, select, sync::Notify};
use np_common::data_io_len::AtomicDataIOLen;

// 开始客户端的所有监听
pub async fn ready() -> Result<(), NpsError> {
    //获取所有启用的端口转发信息
    let active_list = forward_dao::select_enabled(&db::get()).await?;
    for it in active_list {
        //只监听TCP隧道
        accept_forward(it).await?;
    }
    Ok(())
}

// 开始监听某个隧道
pub async fn accept_forward(forward: Forward) -> Result<(), NpsError> {
    tokio::spawn(async move {
        let forward_id = forward.id;
        spawn_start(forward).await;

        //接受到accept结束的通知,说明监听已经停止,可以安全地删除关闭通知器
        forward::FORWARD_LIVE_MAP.lock().await.remove(&forward_id);
    });
    Ok(())
}

async fn spawn_start(forward: Forward) {
    //关闭隧道正在通信的连接
    shutdown(forward.id).await;

    let forward_id = forward.id;
    let data_len = AtomicDataIOLen::from(forward.in_len, forward.out_len);
    let closer = Arc::new(Notify::new());
    let bridge_count = Arc::new(AtomicUsize::new(0));

    //保存关闭通知器
    forward::FORWARD_LIVE_MAP.lock().await.insert(
        forward_id,
        forward::ForwardLive {
            closer: closer.clone(),
            data_len: data_len.clone(),
            bridge_count: bridge_count.clone(),
        },
    );

    let _closer = closer.clone();
    select! {
        // 接收到关闭通知：退出 accept 循环
        _ = _closer.notified() => {
            println!("-->接收到关闭通知：退出 accept 循环");
            return;
        }
        // 接收到全局关闭通知：退出 accept 循环
        _ = application::SHUTDOWN_NOTIFY.notified() => {
            return;
        }
        result = start(
            forward,
            closer,
            data_len,
            bridge_count) =>{
            if let Err(e) = result
            {
                println!("监听隧道发生了错误:{:?}", e);
            }
        }
    }
}

/**
 * 等待代理客户端连接
 */
async fn start(
    forward: Forward, //端口转发信息
    closer: Arc<Notify>,
    data_len: AtomicDataIOLen,
    bridge_count: Arc<AtomicUsize>,
) -> Result<(), NpsError> {
    let tcp_listener = match TcpListener::bind(format!("0.0.0.0:{}", forward.server_port)).await {
        Ok(v) => v,
        Err(e) => {
            forward_dao::set_error(&db::get(), forward.id, format!("监听端口失败:{:?}", e)).await?;
            return Ok(());
        }
    };

    //清除错误消息
    forward_dao::clear_error(&db::get(), forward.id).await?;
    loop {
        let (proxy_tcp, addr) = tcp_listener.accept().await?;
        tcp_bridge::ready(tcp_bridge::TcpBridgeParam {
            ip: addr.ip().to_string(),
            forward_id: forward.id,
            is_stats_traffic: forward.is_stats_traffic,
            target_addr: forward.target_port.clone(),
            proxy_tcp,
            data_len: data_len.clone(),
            closer: closer.clone(),
            bridge_count: bridge_count.clone(),
        })
        .await;
    }
    Ok(())
}

// 关闭监听
// - forward_id 端口转发id
pub async fn shutdown(forward_id: i64) {
    loop {
        //等待隧道代理监听停止,否则可能导致下次监听同一端口失败
        if let Some(forward_live) = forward::FORWARD_LIVE_MAP.lock().await.get(&forward_id) {
            let _ = forward_live.closer.notify_waiters();
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        } else {
            return;
        }
    }
}
