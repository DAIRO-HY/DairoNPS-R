use crate::dao::system_config_dao;
use lib_axum_extract::response::{AppResponse, ResponseExt};
use crate::extension::number::NumberExtension;
use crate::{forward, nps};
use axum::response::sse::{Event, Sse};
use axum::response::{IntoResponse, Response};
use futures::{Stream, TryFutureExt};
use std::{convert::Infallible, time::Duration};
use std::sync::atomic::Ordering;
use tokio_stream::StreamExt;
use lib_np_common::data_io_len::DataIOLen;
use crate::forward::forward_timer;
use crate::nps::nps_timer;

pub async fn index() -> &'static str {
    "Hello, Index!"
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
