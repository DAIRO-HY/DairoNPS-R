use crate::dao::traffic_stats_dao::TrafficStats;
use crate::dao::{traffic_stats_dao, forward_dao, system_config_dao};
use crate::model::data_io_len::DataIOLen;
use crate::{application, forward};
use itertools::Itertools;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::time::sleep;
use np_common::time_util;

/// 准备用来存入数据库的数据缓存，避免频繁操作数据库
pub static INSERT_CACHE_LIST: LazyLock<Mutex<Vec<TrafficStats>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

/// 定时统计端口转发流量
pub fn init() {
    //统计端口转发数据总量
    tokio::spawn(collect_data_len());
}

/// 统计端口转发数据总量
async fn collect_data_len() {
    //上一次统计到的隧道数据长度
    let mut pre_len_map: HashMap<i64, DataIOLen> = HashMap::new();

    // 记录距离上次统计间隔时间
    let mut pre_insert_time: u64 = 0;
    loop {
        sleep(Duration::from_millis(application::ARGS.data_collect_interval)).await;
        collect_data(&mut pre_len_map, &mut pre_insert_time)
            .await
            .unwrap_or_else(|it| {
                //@这里应该写入日志文件，后续开发
                println!("数据统计出错：{:?}", it);
            });
    }
}

/// 收集流量数据
async fn collect_data(
    pre_len_map: &mut HashMap<i64, DataIOLen>,
    pre_insert_time: &mut u64,
) -> Result<(), sqlx::Error> {
    let mut insert_cache_list = INSERT_CACHE_LIST.lock().await;
    *pre_insert_time += application::ARGS.data_collect_interval;

    //当前时间戳
    let now = time_util::current_secs() as i64;
    let forward_live_map = forward::FORWARD_LIVE_MAP.lock().await;
    for (forward_id, forward_live) in forward_live_map.iter() {
        let mut pre_len = match pre_len_map.get_mut(forward_id) {
            Some(it) => it,
            None => {
                //如果该隧道不存在，则添加
                pre_len_map.insert(*forward_id, DataIOLen::default());
                pre_len_map.get_mut(forward_id).unwrap()
            }
        };
        if *pre_len == forward_live.data_len.load() {
            //如果数据总量没有变化，跳过
            // println!("-->数据总量没有变化，跳过");
            continue;
        }

        //每隔STATS_INTERVAL毫秒统计一次数据总量
        let current_data_io = forward_live.data_len.load();
        insert_cache_list.push(TrafficStats {
            forward_id: *forward_id,
            client_id: 0,
            channel_id: 0,
            date: now,
            in_len: (current_data_io.in_len - pre_len.in_len) as i64, //计算数据长度差
            out_len: (current_data_io.out_len - pre_len.out_len) as i64,
        });

        //更新上次统计数量
        *pre_len = current_data_io;
    }
    drop(forward_live_map);
    if *pre_insert_time > application::ARGS.data_collect_insert_interval {
        let mut tx = db::get().begin().await?;

        //批量循环插入数据
        for it in &*insert_cache_list {
            traffic_stats_dao::insert(&mut *tx, it).await?
        }

        //-------------------------------------统计端口转发流量------------------------------------------
        for (forward_id, data_len) in pre_len_map {
            forward_dao::set_data_len(
                &mut *tx,
                *forward_id,
                data_len.in_len as i64,
                data_len.out_len as i64,
            )
            .await?;
        }

        //-------------------------------------统计总流量------------------------------------------
        let total = insert_cache_list
            .iter()
            .map(|it|DataIOLen::from(it.in_len as u64,it.out_len as u64))
            .fold(DataIOLen::default(), |pre, (it)|  pre + it);
        system_config_dao::update_data_io(&mut *tx, total.in_len as i64, total.out_len as i64)
            .await?;

        tx.commit().await?;
        insert_cache_list.clear();
        *pre_insert_time = 0;
    }
    Ok(())
}
