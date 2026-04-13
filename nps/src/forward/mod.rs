use crate::forward::tcp_bridge::TCPBridgeInfo;
use crate::model::data_io_len::AtomicDataIOLen;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};
use tokio::sync::{Mutex, Notify};

pub mod tcp_accept;
pub mod tcp_accept_manager;
pub mod tcp_bridge;
pub mod tcp_bridge_manager;

//端口转发监听信息
pub static FORWARD_LIVE_MAP: LazyLock<Mutex<HashMap<i64, ForwardLive>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// 端口转发监听连接信息
#[derive(Debug)]
pub struct ForwardLive {
    /// 当前流量总和
    pub data_len: AtomicDataIOLen,

    /// 关闭通知
    pub closer: Arc<Notify>,

    /// 正在通信的桥接信息
    pub bridger: Arc<DashMap<u64, TCPBridgeInfo>>,
}

/// 开启转发端口监听
pub fn read() {
    tokio::spawn(async {
        if let Err(e) = tcp_accept_manager::accept().await{
            eprintln!("监听端口转发生了错误:{:?}", e);
        }
    });
}
