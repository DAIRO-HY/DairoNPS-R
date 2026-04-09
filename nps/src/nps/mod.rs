pub mod nps_bridge;
pub mod nps_client;
pub mod nps_pool;
pub mod nps_proxy;
mod nps_timer;
mod nps_error;

use crate::model::data_io_len::AtomicDataIOLen;
use crate::nps::nps_bridge::tcp_bridge::TCPBridgeInfo;
use crate::nps::nps_pool::tcp_pool::TCPPool;
use bytes::Bytes;
use dashmap::DashMap;
use itertools::Itertools;
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::sync::LazyLock;
use serde::Serialize;
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex, Notify};

//隧道穿透信息
pub static CHANNEL_NPS_MAP: LazyLock<Mutex<HashMap<i64, ChannelNPS>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

//客户端连接池
pub static CLIENT_NPS_MAP: LazyLock<Mutex<HashMap<i64, ClientNPS>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// 隧道相关的连接信息
#[derive(Debug)]
pub struct ChannelNPS {
    /// 所属客户端ID
    pub client_id: i64,

    /// 当前流量总和
    pub data_len: AtomicDataIOLen,

    /// 关闭通知
    pub closer: Arc<Notify>,

    /// 正在通信的桥接信息
    pub bridger: Arc<DashMap<u64, TCPBridgeInfo>>,
}

/// 穿透客户端相关的连接信息
pub struct ClientNPS {
    /// 与客户端的tcp连接池
    pub tcp_pool: Vec<TCPPool>,

    /// 与客户端通信的消息发送器
    pub sender: Sender<Bytes>,

    /// 最后一次收到客户端心跳时间
    pub heart_time: Arc<AtomicU64>,
}

// NPS模块初始化
pub fn init() {

    // 启动定时任务
    nps_timer::init();
}
