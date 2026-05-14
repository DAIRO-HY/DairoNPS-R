use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use std::sync::atomic::{AtomicU64, AtomicUsize};
use tokio::sync::{Mutex, Notify};
use lib_np_common::data_io_len::AtomicDataIOLen;

pub mod tcp_accept;
pub mod tcp_bridge;
pub mod forward_timer;

//端口转发监听信息
pub static FORWARD_LIVE_MAP: LazyLock<Mutex<HashMap<i64, ForwardLive>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// 正在通信的桥接信息
pub static FORWARD_BRIDGING_MAP: LazyLock<DashMap<u64, TCPBridging>> =
    LazyLock::new(|| DashMap::new());

/// 端口转发监听连接信息
#[derive(Debug)]
pub struct ForwardLive {
    /// 当前流量总和
    pub data_len: AtomicDataIOLen,

    /// 关闭通知
    pub closer: Arc<Notify>,

    /// 用来统计桥接数量，虽然bridger也可以统计桥接数量，但是当不统计数据流量时bridger将无效
    pub bridge_count: Arc<AtomicUsize>,
}

/// TCP桥接信息
#[derive(Debug,Clone)]
pub struct TCPBridging {

    /// 代理客户端ip地址
    pub ip: String,

    /// 端口转发ID
    pub forward_id: i64,

    /// 流量
    pub data_len: AtomicDataIOLen,

    /// 创建时间(毫秒)
    pub create_time: u64,

    /// 记录最后通信时间(毫秒)
    pub last_rw_time: Arc<AtomicU64>,

    ///关闭监听器
    pub closer: Arc<Notify>,
}

/// 开启转发端口监听
pub fn read() {

    // 定时统计端口转发流量
    forward_timer::init();
    tokio::spawn(async {
        if let Err(e) = tcp_accept::ready().await{
            eprintln!("监听端口转发生了错误:{:?}", e);
        }
    });
}
