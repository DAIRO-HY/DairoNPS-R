use crate::constant::nps_constant;
use crate::dao::channel_data_dao::ChannelData;
use crate::dao::{channel_dao, channel_data_dao, client_dao, system_config_dao};
use crate::extension::number::ToDateFormat;
use crate::model::data_io_len::DataIOLen;
use crate::nps::nps_client::tcp_client::tcp_client_session_manager;
use crate::nps::{CHANNEL_NPS_MAP, CLIENT_NPS_MAP};
use crate::{application, nps};
use itertools::Itertools;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
/// 一些定时任务
use tokio::time::sleep;

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
    //最后一次统计到的隧道数据总量
    let mut last_total_map: HashMap<i64, DataIOLen> = HashMap::new();

    //用来缓存数据统计，避免频繁操作数据库
    let mut channel_data_list: Vec<ChannelData> = Vec::new();

    // 记录距离上次统计间隔时间
    let mut pre_insert_time: u64 = 0;
    loop {
        sleep(Duration::from_millis(application::DATA_COLLECT_INTERVAL)).await;
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
    last_total_map: &mut HashMap<i64, DataIOLen>,
    pre_insert_time: &mut u64,
) -> Result<(), sqlx::Error> {
    *pre_insert_time += application::DATA_COLLECT_INTERVAL;
    let channel_nps_map = CHANNEL_NPS_MAP.lock().await;
    for (channel_id, channel_nps) in channel_nps_map.iter() {
        let last_total = match last_total_map.get(channel_id) {
            Some(v) => v.clone(),
            None => DataIOLen::default(),
        };
        if last_total == channel_nps.data_len.load() {
            //如果数据总量没有变化，跳过
            // println!("-->数据总量没有变化，跳过");
            continue;
        }

        //每隔STATS_INTERVAL毫秒统计一次数据总量
        let data_io = channel_nps.data_len.load();
        channel_data_list.push(ChannelData {
            client_id: channel_nps.client_id,
            channel_id: channel_id.clone(),
            date: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            in_len: data_io.in_len as i64,
            out_len: data_io.out_len as i64,
        });
        last_total_map.insert(*channel_id, data_io);
    }
    drop(channel_nps_map);
    if *pre_insert_time > application::DATA_COLLECT_INSERT_INTERVAL {
        let mut tx = db::get().begin().await?;

        //批量循环插入数据
        for it in &*channel_data_list {
            channel_data_dao::insert(&mut *tx, it).await?
        }

        //得到所有隧道更新之前的流量
        let pre_channel_data_map: HashMap<i64, (i64, i64)> = channel_dao::select_all(&mut *tx)
            .await?
            .iter()
            .map(|it| (it.id, (it.in_len, it.out_len)))
            .collect();

        //-------------------------------------统计隧道流量------------------------------------------
        //按隧道分组，将分组后的最后一条隧道数据更新到数据库表
        let channel_group = channel_data_list
            .iter()
            .into_group_map_by(|it| it.channel_id);

        //本次统计隧道发生变化的流量Map,用来计算本次客户端更新的流量
        let mut current_channel_change_len_map = HashMap::new();
        for (channel_id, list) in channel_group {
            let last = list.last().unwrap();

            let in_len = last.in_len;
            let out_len = last.out_len;

            //更新该隧道的流量总和
            channel_dao::set_data_len(&mut *tx, last.channel_id, in_len, out_len).await?;

            let Some((pre_in, pre_out)) = pre_channel_data_map.get(&channel_id) else {
                continue;
            };

            //计算本次统计隧道发生变化的流量
            current_channel_change_len_map.insert(channel_id, DataIOLen::from(in_len - pre_in, out_len - pre_out));
        }

        //-------------------------------------统计客户端流量------------------------------------------
        //按客户端对应的本次变化数据大小
        let client_2_len = channel_data_list
            .iter()
            .map(|it| (it.client_id, it.channel_id))
            .into_group_map()
            .into_iter()
            .map(|(client_id, mut channel_ids)| {
                channel_ids.dedup(); //由于里面有重复隧道id数据，需要去除重复的隧道id

                //再将该客户端下的所有隧道数据流量求和
                let client_total_len = channel_ids.iter().map(|it| match current_channel_change_len_map.get(it) {
                    Some(v) => v.clone(),
                    None => DataIOLen::default(),
                }).fold(DataIOLen::default(),|pre,it|{
                    pre + it
                });
                (client_id, client_total_len)
            })
            .collect::<HashMap<i64, DataIOLen>>();

        for (client_id, data_len) in &client_2_len {
            client_dao::set_data_len(&mut *tx, client_id.clone(), data_len.in_len as i64, data_len.out_len as i64)
                .await?;
        }

        //-------------------------------------统计总流量------------------------------------------
        let total = client_2_len
            .into_iter()
            .fold(DataIOLen::default(), |pre, (_,it)| pre + it);
        system_config_dao::update_data_io(&mut *tx, total.in_len as i64, total.out_len as i64).await?;

        tx.commit().await?;
        *channel_data_list = Vec::new();
        *pre_insert_time = 0;
        //println!("-->执行了一次统计")
    }
    Ok(())
}

/// 关闭长时间没有心跳的客户端连接
async fn close_not_heart_client() {
    loop {
        sleep(Duration::from_millis(application::HEART_TIME * 2)).await;
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

/// 超时连接池整理
async fn tcp_pool_timeout_check() {
    loop {
        sleep(Duration::from_secs(nps_constant::RECYLE_POOL_TIME_OUT / 2)).await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut client_map = nps::CLIENT_NPS_MAP.lock().await;

        //用来记录连接池被清空的客户端ID,用于请求创建新的连接池
        let mut empty_pool_clients = Vec::new();
        for (client_id, client_nps) in client_map.iter_mut() {
            client_nps.tcp_pool.retain(|it| {
                //连接池超过指定时间,关闭连接
                if now - it.create_time > nps_constant::RECYLE_POOL_TIME_OUT {
                    //连接池超过指定时间,关闭连接
                    // let _ = it.tcp.shutdown();
                    return false;
                }
                return true;
            });
            if client_nps.tcp_pool.len() == 0 {
                //如果连接池被清空，则记录客户端ID,用于请求创建新的连接池,而不是直接在这里请求创建新的连接池,因为这里还持有连接池的锁,如果在这里请求创建新的连接池,可能会导致死锁
                empty_pool_clients.push(*client_id);
            }
        }
        drop(client_map); //释放连接池锁

        //请求添加连接池
        for client_id in empty_pool_clients {
            tcp_client_session_manager::send_tcp_pool_request(
                client_id,
                nps_constant::ADD_POOL_COUNT,
            )
            .await;
        }
    }
}
