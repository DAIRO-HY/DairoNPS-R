use crate::nps::nps_client::nps_session;
use crate::nps_error::NpsError;
use crate::{application, nps};
use np_common::{head_flag, time_util};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

// TCP连接池
pub struct TCPPool {
    pub tcp: TcpStream,

    // 创建时间(毫秒)
    pub create_time:u64,
}

// 获取某个客户端连接池数量
pub async fn get_pool_count(client_id: &i64) -> usize {
    nps::CLIENT_LIVE_MAP
        .lock()
        .await
        .get(&client_id)
        .map_or(0, |it| it.tcp_pool.len())
}

// 添加TCP连接池
pub async fn add(mut tcp: TcpStream) -> Result<(), NpsError> {
    let client_id = tcp.read_i64().await?;
    let mut client_live_map = nps::CLIENT_LIVE_MAP.lock().await;

    //得到客户端连接池列表
    let Some(mut client_live) = client_live_map.get_mut(&client_id) else {
        return Ok(());
    };
    if client_live.tcp_pool.len() >= application::ARGS.max_pool_count {
        // println!("-->客户端: {}连接池已满,拒绝新连接。count: {}", client_id, pools.len());
        //已经达到最大连接数,拒绝新连接
        drop(client_live_map); // 释放锁
        
        //发送连接池已满的标记
        tcp.write_u8(head_flag::POOL_IS_FULL).await?;

        //已经达到最大连接数,拒绝新连接
        tcp.shutdown().await?;
        return Err(NpsError::PoolIsFull);
    }
    let pool = TCPPool {
        create_time: time_util::current_millis(),
        tcp: tcp,
    };
    (client_live.tcp_pool).push(pool);
    Ok(())
}

/**
 * 通过客户端ID获取一个连接
 * @param clientID 客户端ID
 */
async fn get(client_id: i64) -> Option<(TcpStream, usize)> {
    let mut client_live_map = nps::CLIENT_LIVE_MAP.lock().await;

    //得到客户端连接池列表
    let Some(mut client_live) = client_live_map.get_mut(&client_id) else {
        return None;
    };
    let count = client_live.tcp_pool.len();
    if count == 0 {
        return None;
    }

    //从连接池取出并移除最旧的连接，较少连接池中存在过期连接
    Some((client_live.tcp_pool.remove(0).tcp, count))
}

/**
 * 从连接池获取一个连接,并请求添加连接池
 * @param clientID 客户端ID
 */
pub async fn get_and_add_pool(client_id: i64) -> Option<TcpStream> {
    let mut pool_info: Option<(TcpStream, usize)> = None;
    for _ in 0..5 {
        pool_info = get(client_id).await;
        if pool_info.is_some() {
            break;
        }

        //连接池里没有数据，申请创建连接池
        nps_session::send_tcp_pool_request(client_id, application::ARGS.add_pool_count)
            .await;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
    let Some((tcp, count)) = pool_info else {
        return None;
    };

    //申请创建连接池
    nps_session::send_tcp_pool_request(
        client_id,
        if application::ARGS.max_pool_count == count {
            1
        } else {
            2
        },
    ).await;
    Some(tcp)
}