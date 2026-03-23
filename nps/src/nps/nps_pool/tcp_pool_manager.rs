use super::tcp_pool::TCPPool;
use crate::constant::nps_constant;
use crate::nps::nps_client::header_util;
use crate::nps::nps_client::tcp_client::tcp_client_session_manager;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncWriteExt, Result};
use tokio::net::TcpStream;
use crate::nps::POOL_MAP;

pub fn init() {

    // 超时连接池整理
    tokio::spawn(timeout_check());
}

// func init() {
//     go timeoutCheck()
// }
//
// // 当前连接池数量
// func GetPoolCount() int {
//     count := 0
//     poolLock.Lock()
//     for _, pools := range poolMap {
//         count += len(*pools)
//     }
//     poolLock.Unlock()
//     return count
// }

// 获取某个客户端连接池数量
pub async fn get_pool_count(client_id: &i64) -> usize {
    return POOL_MAP.get().unwrap().lock().await.get(&client_id).map_or(0, |it| it.len());
}

/**
 * 为客户端创建一个空的连接池
 * @param client_id 客户端ID
 */
pub async fn init_empty_pool_by_client(client_id: i64) {
    //移除旧的连接池并创建新的连接池
    POOL_MAP.get().unwrap().lock().await.insert(client_id, Vec::new());
}

// 添加TCP连接池
// clientSocket tcp连接
pub async fn add(mut tcp: TcpStream) -> Result<()> {
    //从头部信息中得到客户端id
    let client_id_str = header_util::get_header(&mut tcp).await?;
    let client_id: i64 = client_id_str.parse().unwrap();

    let mut pool_map = POOL_MAP.get().unwrap().lock().await;

    //得到客户端连接池列表
    let Some(pools) = pool_map.get_mut(&client_id) else{
        return Ok(());
    };
    if pools.len() >= nps_constant::MAX_POOL_COUNT {
        println!("-->客户端: {}连接池已满,拒绝新连接。count: {}", client_id, pools.len());
        //已经达到最大连接数,拒绝新连接
        drop(pool_map); // 释放锁

        //已经达到最大连接数,拒绝新连接
        tcp.shutdown().await?;
        return Result::Err(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "已经达到最大连接数,拒绝新连接",
        ));
    }
    let pool = TCPPool {
        create_time: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        tcp: tcp,
    };
    pools.push(pool);
    Ok(())
}

/**
 * 通过客户端ID获取一个连接
 * @param clientID 客户端ID
 */
async fn get(client_id:i64)-> Option<(TcpStream,usize)> {
    let mut pool_map = POOL_MAP.get().unwrap().lock().await;
    let Some(pools) = pool_map.get_mut(&client_id) else {
        return None;
    };
    let count = pools.len();
    if count == 0 {
        return None;
    }

    //从连接池取出并移除最后一次添加到连接池的连接
    // return Some(pools.pop().unwrap().tcp);

    //从连接池取出并移除最旧的连接，较少连接池中存在过期连接
    return Some((pools.remove(0).tcp, count));
}

/**
 * 从连接池获取一个连接,并请求添加连接池
 * @param clientID 客户端ID
 */
pub async fn get_and_add_pool(client_id: i64) -> Option<TcpStream> {
     let mut pool_info: Option<(TcpStream,usize)> = None;
     for _ in 0..5 {
        pool_info = get(client_id).await;
        if pool_info.is_some() {
            break;
        }

        //连接池里没有数据，申请创建连接池
        tcp_client_session_manager::send_tcp_pool_request(client_id, nps_constant::ADD_POOL_COUNT).await;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    let Some((tcp, count)) = pool_info else {
        return None;
    };

    //申请创建连接池
    tokio::spawn(tcp_client_session_manager::send_tcp_pool_request(client_id, if(nps_constant::MAX_POOL_COUNT == count){1}else{2}));
    println!("-->当前连接池数量: {} ", count);
    return Some(tcp);
}

// /**
//  * 发起连接池申请请求
//  * 每取走一个连接,则请求创建2个新的连接,直到达到最大连接数
//  * @param clientID 客户端ID
//  */
// async fn pool_request(client_id: u64, count: u8) {
//     let mut pool_map = POOL_MAP.get().unwrap().lock().await;

//     //得到客户端连接池列表
//     let pools = pool_map.get(&client_id);
//     if pools.is_none() {
//         return;
//     }
//     let pools = pools.unwrap();
//     if pools.len() >= crate::constant::nps_constant::MAX_POOL_COUNT {//已经达到最大连接数
//         return;
//     }
//     tcp_client_session_manager::send_tcp_pool_request(client_id, count).await;
// }

/**
 * 移除某个客户端所有的连接池
 * @param clientID 客户端ID
 */
pub async fn shutdown_by_client(client_id: i64) {
    POOL_MAP.get().unwrap().lock().await.remove(&client_id);
    println!("tcp连接池被关闭了...");
}

// 超时连接池整理
async fn timeout_check() {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(nps_constant::RECYLE_POOL_TIME_OUT / 2)).await;
        let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
        let mut pool_map = POOL_MAP.get().unwrap().lock().await;

        //用来记录连接池被清空的客户端ID,用于请求创建新的连接池
        let mut empty_pool_clients = Vec::new();
        for (client_id, pools) in pool_map.iter_mut() {
            pools.retain(|it| {

                //连接池超过指定时间,关闭连接
                if now - it.create_time > nps_constant::RECYLE_POOL_TIME_OUT {
                    //连接池超过指定时间,关闭连接
                    // let _ = it.tcp.shutdown();
                    return false;
                }
                return true;
            });
            if pools.len() == 0 {//如果连接池被清空，则记录客户端ID,用于请求创建新的连接池,而不是直接在这里请求创建新的连接池,因为这里还持有连接池的锁,如果在这里请求创建新的连接池,可能会导致死锁
                empty_pool_clients.push(*client_id);
            }
        }
        drop(pool_map);//释放连接池锁

        //请求添加连接池
        for client_id in empty_pool_clients {
            tcp_client_session_manager::send_tcp_pool_request(client_id, nps_constant::ADD_POOL_COUNT).await;
        }
    }
}

// // 超时连接池整理
// func timeoutCheck() {
//     for {
//         time.Sleep(NPSConstant.RECYLE_POOL_TIME_OUT * time.Millisecond)
//
//         //当前时间戳秒
//         now := time.Now().UnixMilli()
//         poolLock.Lock()
//         for clientId, pools := range poolMap { //遍历所有客户端的连接池
//             poolList := *pools
//             poolSize := len(poolList)
//             for i := poolSize - 1; i > -1; i-- {
//                 pool := (*pools)[i]
//                 if now-pool.CreateTime > NPSConstant.RECYLE_POOL_TIME_OUT { //连接池超过指定时间
//                     pool.PoolTCP.Close()
//                     poolList = poolList[0:i]
//                 }
//             }
//             if len(poolList) == 0 { //如果连接池被清空，则请求创建一个新的连接池
//                 Csmi.SendTCPPoolRequest(clientId, 1)
//             }
//             *pools = poolList
//         }
//         poolLock.Unlock()
//     }
// }
