use crate::dao::system_config_dao;
use crate::extension::ResponseEmptyExt;
use crate::extension::number::ToDataSize;
use crate::nps;
use axum::response::{IntoResponse, Response};
use axum::{
    Json, Router,
    response::sse::{Event, Sse},
    routing::get,
};
use futures::{Stream, TryFutureExt};
use std::{convert::Infallible, time::Duration};
use sysinfo::System;
use tokio_stream::StreamExt;

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
        tokio::time::sleep(Duration::from_secs(1)).await;
        let data = get_data().await;
        let json = serde_json::to_string(&data).unwrap();
        Ok(Event::default().data(json))
    });
    Sse::new(stream)
}

fn get_used_memory() -> String {
    let mut sys = System::new_all();
    sys.refresh_all();
    let pid = sysinfo::get_current_pid().unwrap();
    sys.process(pid).unwrap().memory().data_size()
}

// 页面初始化
async fn get_data() -> model::NPSStatus {
    let system_config = system_config_dao::get(&db::get()).await.unwrap_or_default();
    //
    // // 获取内存使用情况
    // var memStats runtime.MemStats
    // runtime.ReadMemStats(&memStats)
    let client_nps_map = nps::CLIENT_NPS_MAP.lock().await;
    let online_client_count = client_nps_map.len();
    let tcp_pool_count = client_nps_map
        .iter()
        .fold(0, |p, (_, it)| p + it.tcp_pool.len());
    drop(client_nps_map);

    let channel_nps_map = nps::CHANNEL_NPS_MAP.lock().await;
    let channel_count = channel_nps_map.len();
    let tcp_bridge_count = channel_nps_map
        .iter()
        .fold(0, |p, (_, it)| p + it.bridger.len());
    drop(channel_nps_map);

    model::NPSStatus {
        channel_count,      //当前正在代理数
        online_client_count, //在线客户端数量
        tcp_bridge_count, //当前TCP桥接数
        tcp_pool_count, //当前TCP连接池
        // UdpBridgeCount:     udp_bridge.GetBridgeCount(),          //当前UDP桥接数
        // UdpPoolCount:       udp_pool.GetPoolCount(),              //当前UDP连接池
        in_bytes: system_config.in_data.data_size(), //入网流量
        out_bytes: system_config.out_data.data_size(), //出网流量
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
        pub in_bytes: String,

        // 出网流量
        pub out_bytes: String,
        // // 当前正在代理服务数
        // pub ForwardCount: usize,
        //
        // // 代理服务会话数
        // pub ForwardBridgeCount: usize,
        //
        // pub NumGoroutine: usize,    //当前协程数
        // pub Memory: String, //内存分配
    }
}
