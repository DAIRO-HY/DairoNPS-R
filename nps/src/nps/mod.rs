pub mod nps_bridge;
pub mod nps_client;
pub mod nps_pool;
pub mod nps_proxy;
pub mod nps_timer;
pub mod security_util;

use crate::model::data_io_len::AtomicDataIOLen;
use bytes::Bytes;
use dashmap::DashMap;
use itertools::Itertools;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize};
use std::sync::Arc;
use std::sync::LazyLock;
use serde::Serialize;
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex, Notify};
use crate::nps::nps_bridge::tcp_bridge::TCPBridgeInfo;
use crate::nps::nps_pool::tcp_pool::TCPPool;

//隧道穿透信息
pub static CHANNEL_LIVE_MAP: LazyLock<Mutex<HashMap<i64, ChannelLive>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

//客户端连接池
pub static CLIENT_LIVE_MAP: LazyLock<Mutex<HashMap<i64, ClientLive>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// 隧道监听相关的连接信息
#[derive(Debug)]
pub struct ChannelLive {
    /// 所属客户端ID
    pub client_id: i64,

    /// 当前流量总和
    pub data_len: AtomicDataIOLen,

    /// 关闭通知
    pub closer: Arc<Notify>,

    /// 正在通信的桥接信息
    pub bridge_map: Arc<DashMap<u64, TCPBridgeInfo>>,

    /// 用来统计桥接数量，虽然bridger也可以统计桥接数量，但是当不统计数据流量时bridger将无效
    pub bridge_count: Arc<AtomicUsize>,
}

/// 穿透客户端监听相关的连接信息
pub struct ClientLive {
    /// 与客户端的tcp连接池
    pub tcp_pool: Vec<TCPPool>,

    /// 与客户端通信的消息发送器
    pub sender: Sender<Bytes>,

    /// 最后一次收到客户端心跳时间
    pub heart_time: Arc<AtomicU64>,
}

// NPS模块开启
pub fn ready() {

    // 启动定时任务
    nps_timer::init();
    tokio::spawn(async {
        if let Err(e) = nps_client::tcp_client::tcp_client_accept::accept().await{
            eprintln!("监听客户端发生了错误:{:?}", e);
        }
    });
}
