pub mod nps_bridge;
pub mod nps_client;
pub mod nps_pool;
pub mod nps_proxy;

use crate::dao::channel_data_dao;
use crate::dao::channel_data_dao::ChannelData;
use crate::model::data_total::DataTotal;
use crate::nps::nps_bridge::tcp_bridge::TCPBridgeInfo;
use crate::nps::nps_client::tcp_client::tcp_client_session_manager;
use crate::nps::nps_pool::tcp_pool::TCPPool;
use crate::{application, nps};
use bytes::Bytes;
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::Sender;
use tokio::sync::{Mutex, Notify};
use tokio::time::Duration;

//隧道穿透信息
pub static CHANNEL_NPS_MAP: LazyLock<Mutex<HashMap<i64, ChannelNPS>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// //每个隧道ID对应一个关闭通知器,用于通知TCPProxyAccept停止监听
// pub static CHANNEL_CLOSE_NOTIFY: LazyLock<Mutex<HashMap<i64, Arc<Notify>>>> =
//     LazyLock::new(|| Mutex::new(HashMap::new()));
//
// //隧道数据总量
// pub static CHANNEL_DATA_TOTAL: LazyLock<Mutex<HashMap<i64, DataTotal>>> =
//     LazyLock::new(|| Mutex::new(HashMap::new()));
//
// //当前正在通信的桥接信息
// pub static BRIDGE_INFO: LazyLock<Mutex<HashMap<i64, Arc<DashMap<u64, TCPBridgeInfo>>>>> =
//     LazyLock::new(|| Mutex::new(HashMap::new()));

//客户端连接池
pub static CLIENT_NPS_MAP: LazyLock<Mutex<HashMap<i64, ClientNPS>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

// //客户端连接池
// pub static POOL_MAP: LazyLock<Mutex<HashMap<i64, Vec<TCPPool>>>> =
//     LazyLock::new(|| Mutex::new(HashMap::new()));
//
// //客户端ID对应的Socket连接
// pub static CLIENT_SESSION: LazyLock<Mutex<HashMap<i64, Sender<Bytes>>>> =
//     LazyLock::new(|| Mutex::new(HashMap::new()));

const INSERT_INTERVAL: u64 = 60000; //插入数据间隔，单位毫秒
const STATS_INTERVAL: u64 = 1000; //统计间隔，单位毫秒

/// 隧道相关的连接信息
pub struct ChannelNPS {
    /// 所属客户端ID
    pub client_id: i64,

    /// 当前流量总和
    pub data_total: DataTotal,

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
    // // 初始化客户端会话管理器
    // let _ = CLIENT_SESSION.set(Mutex::new(HashMap::new()));

    // // 初始化连接池
    // let _ = POOL_MAP.set(Mutex::new(HashMap::new()));

    // // 初始化隧道关闭通知器
    // let _ = CHANNEL_CLOSE_NOTIFY.set(Mutex::new(HashMap::new()));

    // // 初始化隧道数据总量
    // let _ = CHANNEL_DATA_TOTAL.set(Mutex::new(HashMap::new()));

    // // 初始化桥接信息
    // let _ = BRIDGE_INFO.set(Mutex::new(HashMap::new()));

    //初始化连接池模块
    nps_pool::tcp_pool_manager::init();

    //统计隧道数据总量
    tokio::spawn(channel_collect_data());

    //关闭长时间没有心跳的客户端连接
    tokio::spawn(close_not_heart_client());
}

// 统计隧道数据总量
async fn channel_collect_data() {
    //最后一次统计到的隧道数据总量
    let mut last_total_map: HashMap<i64, DataTotal> = HashMap::new();

    //用来缓存数据统计，避免频繁操作数据库
    let mut channel_data_list: Vec<ChannelData> = Vec::new();

    // 记录距离上次统计间隔时间
    let mut pre_insert_time: u64 = 0;
    loop {
        tokio::time::sleep(Duration::from_millis(STATS_INTERVAL)).await;
        collect_data(
            &mut channel_data_list,
            &mut last_total_map,
            &mut pre_insert_time,
        )
        .await
        .unwrap_or_else(|it| {
            //@这里应该写入日志文件，后续开发
            println!("数据统计出错：{:?}", it);
        });
    }
}

/// 收集流量数据
async fn collect_data(
    channel_data_list: &mut Vec<ChannelData>,
    last_total_map: &mut HashMap<i64, DataTotal>,
    pre_insert_time: &mut u64,
) -> Result<(), sqlx::Error> {
    *pre_insert_time += STATS_INTERVAL;

    let channel_nps_map = CHANNEL_NPS_MAP.lock().await;
    for (channel_id, channel_nps) in channel_nps_map.iter() {
        if let Some(last_total) = last_total_map.get(channel_id)
            && *last_total == channel_nps.data_total
        {
            //如果数据总量没有变化，跳过
            break;
        }

        //每隔STATS_INTERVAL毫秒统计一次数据总量
        let in_data = channel_nps.data_total.load_in();
        let out_data = channel_nps.data_total.load_out();
        channel_data_list.push(ChannelData {
            channel_id: channel_id.clone(),
            date: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            in_data: in_data as i64,
            out_data: out_data as i64,
        });
        last_total_map.insert(*channel_id, DataTotal::from(in_data, out_data));
    }
    drop(channel_nps_map);
    if *pre_insert_time > INSERT_INTERVAL {
        let mut tx = db::get().begin().await?;

        //批量循环插入数据
        for it in &*channel_data_list {
            channel_data_dao::insert(&mut *tx, it).await?
        }
        tx.commit().await?;
        *channel_data_list = Vec::new();
        *pre_insert_time = 0;
        println!("-->执行了一次统计")
    }
    Ok(())
}

/// 关闭长时间没有心跳的客户端连接
async fn close_not_heart_client() {
    loop{
        tokio::time::sleep(Duration::from_millis(application::HEART_TIME * 2)).await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let not_heart_client_id: Vec<i64> = CLIENT_NPS_MAP
            .lock()
            .await
            .iter()
            .filter_map(|(client_id, it)| -> Option<i64> {
                if (now - it.heart_time.load(Ordering::Relaxed)) > application::HEART_TIME * 2 {
                    //指定时间内没有心跳
                    Some(client_id.clone())
                } else {
                    None
                }
            })
            .collect();
        for it in not_heart_client_id {
            let _ = tcp_client_session_manager::shutdown(it).await;
        }
    }
}