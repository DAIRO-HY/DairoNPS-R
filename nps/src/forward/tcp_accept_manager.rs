use crate::dao::forward_dao;
use crate::dao::forward_dao::Forward;
use crate::forward;
use crate::forward::tcp_accept;
use crate::model::data_io_len::AtomicDataIOLen;
use crate::nps_error::NpsError;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;
use tokio::{net::TcpListener, sync::Notify};

// 开始客户端的所有监听
pub async fn accept() -> Result<(), NpsError> {
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
    //关闭隧道正在通信的连接
    shutdown(forward.id).await;
    let tcp_listener = match TcpListener::bind(format!("0.0.0.0:{}", forward.server_port)).await {
        Ok(v) => v,
        Err(e) => {
            forward_dao::set_error(&db::get(), forward.id, format!("监听端口失败:{:?}", e)).await?;
            return Ok(());
        }
    };

    //清除错误消息
    forward_dao::clear_error(&db::get(), forward.id).await?;

    let forward_id = forward.id;
    let data_len = AtomicDataIOLen::from(forward.in_len, forward.out_len);
    let closer = Arc::new(Notify::new());
    let bridger = Arc::new(DashMap::new());
    let bridge_count = Arc::new(AtomicUsize::new(0));

    //保存关闭通知器
    forward::FORWARD_LIVE_MAP.lock().await.insert(
        forward_id,
        forward::ForwardLive {
            closer: closer.clone(),
            data_len: data_len.clone(),
            bridger: bridger.clone(),
            bridge_count: bridge_count.clone(),
        },
    );
    tokio::spawn(async move {
        if let Err(e) = tcp_accept::accept(
            forward,
            tcp_listener,
            closer,
            data_len,
            bridger,
            bridge_count,
        )
        .await
        {
            println!("监听隧道发生了错误:{:?}", e);
        }

        //接受到accept结束的通知,说明监听已经停止,可以安全地删除关闭通知器
        forward::FORWARD_LIVE_MAP.lock().await.remove(&forward_id);
    });
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
