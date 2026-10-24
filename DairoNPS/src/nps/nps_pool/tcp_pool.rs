use crate::nps::nps_client;
use crate::nps::nps_client::nps_session;
use crate::nps_error::NpsError;
use crate::{application, nps};
use lib_np_common::{head_flag, time_util};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{Duration, Instant};

/// 连接池为空时,代理连接最长等待时间
const POOL_WAIT_TIMEOUT: Duration = Duration::from_secs(5);

/// 连接池为空时,重复申请创建连接池的间隔时间
const POOL_REQUEST_INTERVAL: Duration = Duration::from_secs(1);

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
    //读取客户端ID(设置超时,避免恶意连接)
    let client_id = tokio::time::timeout(nps_client::HANDSHAKE_TIMEOUT, tcp.read_i64()).await??;
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

    //唤醒一个正在等待连接池的代理连接
    client_live.tcp_pool_notify.notify_one();
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
    //最长等待时间
    let deadline = Instant::now() + POOL_WAIT_TIMEOUT;

    //下次申请创建连接池的时间
    let mut next_request_time = Instant::now();
    let (tcp, count) = loop {
        //连接池通知器,客户端已经离线时直接返回
        let notify = {
            let client_live_map = nps::CLIENT_LIVE_MAP.lock().await;
            let Some(client_live) = client_live_map.get(&client_id) else {
                return None;
            };
            client_live.tcp_pool_notify.clone()
        };

        //必须在检查连接池之前就开始监听通知,否则在检查之后、等待之前加入的连接会错过通知
        let notified = notify.notified();
        tokio::pin!(notified);
        notified.as_mut().enable();
        if let Some(pool_info) = get(client_id).await {
            break pool_info;
        }
        let now = Instant::now();
        if now >= deadline {
            return None;
        }
        if now >= next_request_time {
            //连接池里没有数据，申请创建连接池
            nps_session::send_tcp_pool_request(client_id, application::ARGS.add_pool_count)
                .await;

            //超过这个时间还没等到新的连接,可能是创建失败了,需要再申请一次
            next_request_time = now + POOL_REQUEST_INTERVAL;
        }

        //等待新的连接加入连接池,而不是固定等待1秒
        let _ = tokio::time::timeout_at(next_request_time.min(deadline), notified).await;
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