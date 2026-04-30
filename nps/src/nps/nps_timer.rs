use crate::constant::nps_constant;
use crate::dao::traffic_stats_dao::TrafficStats;
use crate::dao::{channel_dao, traffic_stats_dao, client_dao, system_config_dao};
use crate::model::data_io_len::DataIOLen;
use crate::nps::{CHANNEL_LIVE_MAP, CLIENT_LIVE_MAP};
use crate::{application, nps};
use itertools::Itertools;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::LazyLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
/// 一些定时任务
use tokio::time::sleep;
use np_common::{head_flag, time_util};
use crate::nps::nps_client::nps_session;

//准备用来存入数据库的数据缓存，避免频繁操作数据库
pub static INSERT_CACHE_LIST: LazyLock<Mutex<Vec<TrafficStats>>> = LazyLock::new(|| Mutex::new(Vec::new()));

pub fn init() {
    //统计隧道数据总量
    tokio::spawn(channel_collect_data());

    //关闭长时间没有心跳的客户端连接
    tokio::spawn(close_not_heart_client());

    //超时连接池整理
    tokio::spawn(tcp_pool_timeout_check());
}

/// 统计隧道数据总量
async fn channel_collect_data() {
    //上一次统计到的隧道数据长度
    let mut pre_channel_len_map: HashMap<i64, DataIOLen> = HashMap::new();

    // 记录距离上次统计间隔时间
    let mut pre_insert_time: u64 = 0;
    loop {
        sleep(Duration::from_millis(application::DATA_COLLECT_INTERVAL)).await;
        collect_data(
            &mut pre_channel_len_map,
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
    pre_channel_len_map: &mut HashMap<i64, DataIOLen>,
    pre_insert_time: &mut u64,
) -> Result<(), sqlx::Error> {
    let mut insert_cache_list = INSERT_CACHE_LIST.lock().await;
    *pre_insert_time += application::DATA_COLLECT_INTERVAL;

    //当前时间戳
    let now = time_util::current_secs() as i64;
    let channel_live_map = CHANNEL_LIVE_MAP.lock().await;
    for (channel_id, channel_live) in channel_live_map.iter() {
        let mut pre_len = match pre_channel_len_map.get_mut(channel_id) {
            Some(it) => it,
            None => {
                //如果该隧道不存在，则添加
                pre_channel_len_map.insert(*channel_id, DataIOLen::default());
                pre_channel_len_map.get_mut(channel_id).unwrap()
            }
        };
        if *pre_len == channel_live.data_len.load() {
            //如果数据总量没有变化，跳过
            // println!("-->数据总量没有变化，跳过");
            continue;
        }

        //每隔STATS_INTERVAL毫秒统计一次数据总量
        let current_data_io = channel_live.data_len.load();
        insert_cache_list.push(TrafficStats {
            forward_id:0,
            client_id: channel_live.client_id,
            channel_id: channel_id.clone(),
            date: now,
            in_len: (current_data_io.in_len - pre_len.in_len) as i64, //计算数据长度差
            out_len: (current_data_io.out_len - pre_len.out_len) as i64,
        });

        //更新上次统计数量
        *pre_len = current_data_io;
    }
    drop(channel_live_map);
    if *pre_insert_time > application::DATA_COLLECT_INSERT_INTERVAL {
        let mut tx = db::get().begin().await?;

        //批量循环插入数据
        for it in &*insert_cache_list {
            traffic_stats_dao::insert(&mut *tx, it).await?
        }

        //-------------------------------------统计隧道流量------------------------------------------
        for (channel_id, data_len) in pre_channel_len_map {
            channel_dao::set_data_len(
                &mut *tx,
                *channel_id,
                data_len.in_len as i64,
                data_len.out_len as i64,
            )
            .await?;
        }

        //-------------------------------------统计客户端流量------------------------------------------
        //按客户端对应的本次变化数据大小
        let client_2_len = insert_cache_list
            .iter()
            // .map(|it| (it.client_id, it.channel_id))
            .into_grouping_map_by(|it| it.client_id)
            .fold(DataIOLen::default(), |pre, _, it| {
                DataIOLen::from(
                    pre.in_len + it.in_len as u64,
                    pre.out_len + it.out_len as u64,
                )
            });
        for (client_id, data_len) in &client_2_len {
            client_dao::set_data_len(
                &mut *tx,
                *client_id,
                data_len.in_len as i64,
                data_len.out_len as i64,
            )
            .await?;
        }

        //-------------------------------------统计总流量------------------------------------------
        let total = client_2_len
            .into_iter()
            .fold(DataIOLen::default(), |pre, (_, it)| pre + it);
        system_config_dao::update_data_io(&mut *tx, total.in_len as i64, total.out_len as i64)
            .await?;

        tx.commit().await?;
        insert_cache_list.clear();
        *pre_insert_time = 0;
        //println!("-->执行了一次统计")
    }
    Ok(())
}

/// 关闭长时间没有心跳的客户端连接
async fn close_not_heart_client() {
    loop {
        sleep(Duration::from_millis(application::HEART_TIME * 2)).await;
        let now = time_util::current_millis();
        let not_heart_client_id: Vec<i64> = CLIENT_LIVE_MAP
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
            let _ = nps_session::shutdown(it).await;
        }
    }
}

/// 超时连接池整理
async fn tcp_pool_timeout_check() {
    loop {
        sleep(Duration::from_secs(nps_constant::RECYLE_POOL_TIME_OUT / 2)).await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut client_map = nps::CLIENT_LIVE_MAP.lock().await;

        //用来记录连接池被清空的客户端ID,用于请求创建新的连接池
        let mut empty_pool_clients = Vec::new();
        for (client_id, client_live) in client_map.iter_mut() {
            client_live.tcp_pool.retain_mut(|it| {
                //连接池超过指定时间,关闭连接
                if now - it.create_time > nps_constant::RECYLE_POOL_TIME_OUT {
                    //连接池超过指定时间,关闭连接
                    return false;
                }
                return true;
            });
            if client_live.tcp_pool.len() == 0 {
                //如果连接池被清空，则记录客户端ID,用于请求创建新的连接池,而不是直接在这里请求创建新的连接池,因为这里还持有连接池的锁,如果在这里请求创建新的连接池,可能会导致死锁
                empty_pool_clients.push(*client_id);
            }
        }
        drop(client_map); //释放连接池锁

        //请求添加连接池
        for client_id in empty_pool_clients {
            nps_session::send_tcp_pool_request(
                client_id,
                nps_constant::ADD_POOL_COUNT,
            )
            .await;
        }
    }
}
