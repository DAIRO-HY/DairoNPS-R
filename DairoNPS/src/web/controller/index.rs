use crate::dao::system_config_dao;
use crate::extension::ResponseEmptyExt;
use crate::extension::number::NumberExtension;
use crate::model::data_io_len::DataIOLen;
use crate::{forward, nps};
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Response};
use futures::{Stream, TryFutureExt};
use std::{convert::Infallible, time::Duration};
use std::sync::atomic::Ordering;
use tokio_stream::StreamExt;
use crate::forward::forward_timer;
use crate::nps::nps_timer;

pub async fn index() -> &'static str {
    "Hello, Index!"
}
pub async fn test() -> Response {
    let data = get_data().await;
    Response::json(data)
}

///　获取系统运行状态
pub async fn get_nps_status() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = tokio_stream::iter(0..).then(|i| async move {
        if i > 0 {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        let data = get_data().await;
        let json = serde_json::to_string(&data).unwrap();
        Ok(Event::default().data(json))
    });
    Sse::new(stream)
}

// 页面初始化
async fn get_data() -> model::NPSStatus {
    //系统配置
    let system_config = system_config_dao::get(&db::get()).await.unwrap_or_default();

    //系统总流量
    let system_len = DataIOLen::from(system_config.in_len,system_config.out_len);

    //隧道等待写入部分数据大小
    let channel_cache_len =
        nps_timer::INSERT_CACHE_LIST
            .lock()
            .await
            .iter()
            .fold(DataIOLen::default(), |pre, it| {
                DataIOLen::from(
                    pre.in_len + it.in_len as u64,
                    pre.out_len + it.out_len as u64,
                )
            });

    //端口转发等待写入部分数据大小
    let forward_cache_len =
        forward_timer::INSERT_CACHE_LIST
            .lock()
            .await
            .iter()
            .fold(DataIOLen::default(), |pre, it| {
                DataIOLen::from(
                    pre.in_len + it.in_len as u64,
                    pre.out_len + it.out_len as u64,
                )
            });
    let total_len = system_len + channel_cache_len + forward_cache_len;

    let client_live_map = nps::CLIENT_LIVE_MAP.lock().await;
    let online_client_count = client_live_map.len();
    let tcp_pool_count = client_live_map
        .iter()
        .fold(0, |p, (_, it)| p + it.tcp_pool.len());
    drop(client_live_map);

    let channel_live_map = nps::CHANNEL_LIVE_MAP.lock().await;
    let channel_count = channel_live_map.len();
    let tcp_bridge_count = channel_live_map
        .iter()
        .fold(0, |p, (_, it)| p + it.bridge_count.load(Ordering::Relaxed));
    drop(channel_live_map);

    let forward_live_map = forward::FORWARD_LIVE_MAP.lock().await;
    let forward_count = forward_live_map.len();
    let forward_bridge_count = forward_live_map
        .iter()
        .fold(0, |p, (_, it)| p + it.bridge_count.load(Ordering::Relaxed));
    drop(forward_live_map);

    model::NPSStatus {
        channel_count,        //当前正在代理数
        online_client_count,  //在线客户端数量
        tcp_bridge_count,     //当前TCP桥接数
        tcp_pool_count,       //当前TCP连接池
        forward_count,        //端口转发监听数量
        forward_bridge_count, //端口转发会话数
        in_len: total_len.in_len.data_size(), //入网流量
        out_len: total_len.out_len.data_size(), //出网流量
    }
}

mod model {
    use serde::{Deserialize, Serialize};

    ///NPS运行状态
    #[derive(Debug, Default, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct NPSStatus {
        // 正在监听的隧道
        pub channel_count: usize,

        // 在线客户端数量
        pub online_client_count: usize,

        // 桥接通信中的数量
        pub tcp_bridge_count: usize,

        // 当前TCP连接池
        pub tcp_pool_count: usize,

        // // 当前UDP会话数
        // pub UdpBridgeCount: usize,
        //
        // // 当前UDP连接池
        // pub UdpPoolCount: usize,

        // 入网流量
        pub in_len: String,

        // 出网流量
        pub out_len: String,

        // 端口转发监听数量
        pub forward_count: usize,

        // 端口转发会话数
        pub forward_bridge_count: usize,
        // pub NumGoroutine: usize,    //当前协程数
        // pub Memory: String, //内存分配
    }
}
