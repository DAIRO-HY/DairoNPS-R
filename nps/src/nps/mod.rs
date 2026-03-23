use std::collections::HashMap;
use std::sync::{Arc, OnceLock};
use bytes::Bytes;
use dashmap::DashMap;
use tokio::sync::{Mutex,MutexGuard, Notify};
use tokio::sync::mpsc::Sender;
use crate::model::data_total::DataTotal;
use crate::nps::nps_bridge::tcp_bridge::TCPBridgeInfo;
use crate::nps::nps_pool::tcp_pool::TCPPool;

pub mod nps_bridge;
pub mod nps_client;
pub mod nps_pool;
pub mod nps_proxy;


//每个隧道ID对应一个关闭通知器,用于通知TCPProxyAccept停止监听
pub static CHANNEL_CLOSE_NOTIFY: OnceLock<Mutex<HashMap<i64, Arc<Notify>>>> = OnceLock::new();

//隧道数据总量
pub static CHANNEL_DATA_TOTAL: OnceLock<Mutex<HashMap<i64, DataTotal>>> = OnceLock::new();

//客户端连接池
pub static POOL_MAP: OnceLock<Mutex<HashMap<i64, Vec<TCPPool>>>> = OnceLock::new();

//客户端ID对应的Socket连接
pub static CLIENT_SESSION: OnceLock<Mutex<HashMap<i64, Sender<Bytes>>>> = OnceLock::new();

//当前正在通信的桥接信息
pub static BRIDGE_INFO: OnceLock<Mutex<HashMap<i64, Arc<DashMap<u64,TCPBridgeInfo>>>>> = OnceLock::new();

// NPS模块初始化
pub fn init() {

    // 初始化客户端会话管理器
    let _ = CLIENT_SESSION.set(Mutex::new(HashMap::new()));

    // 初始化连接池
    let _ = POOL_MAP.set(Mutex::new(HashMap::new()));

    // 初始化隧道关闭通知器
    let _ = CHANNEL_CLOSE_NOTIFY.set(Mutex::new(HashMap::new()));

    // 初始化隧道数据总量
    let _ = CHANNEL_DATA_TOTAL.set(Mutex::new(HashMap::new()));

    // 初始化桥接信息
    let _ = BRIDGE_INFO.set(Mutex::new(HashMap::new()));

    //初始化连接池模块
    crate::nps::nps_pool::tcp_pool_manager::init();
    
    //初始化隧道代理模块
    crate::nps::nps_proxy::tcp_proxy_manager::init();
}
