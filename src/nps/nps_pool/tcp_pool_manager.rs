use super::tcp_pool::TCPPool;
use crate::nps::nps_client::header_util;
use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncWriteExt, Result};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/**
 * 客户端连接池
 */
static POOL_MAP: OnceLock<Mutex<HashMap<u64, Vec<TCPPool>>>> = OnceLock::new();
pub fn init() {
    // 初始化连接池
    // POOL_MAP.set(Mutex::new(HashMap::new())).unwrap();
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
//
// /**
//  * 为客户端创建一个空的连接池
//  * @param clientID 客户端ID
//  */
// func InitEmptyPoolByClient(clientID int) {
//     poolLock.Lock()
//     if poolMap[clientID] != nil {
//         poolLock.Unlock()
//         return
//     }
//
//     //创建连接池列表
//     poolMap[clientID] = &([]*TCPPool{})
//     poolLock.Unlock()
// }

// 添加TCP连接池
// clientSocket tcp连接
pub async fn add(mut tcp: TcpStream) -> Result<()> {


    //从头部信息中得到客户端id
    let client_id_str = header_util::get_header(&mut tcp).await?;
    let client_id: u64 = client_id_str.parse().unwrap();

    let mut map = POOL_MAP.get().unwrap().lock().await;
    let mut pools = map.get(&client_id).unwrap();
    if pools.len() >= crate::constant::nps_constant::MAX_POOL_COUNT {
        drop(map);

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
    drop(map);



    // let map = POOL_MAP.get_or_init(|| Mutex::new(HashMap::new()));
    //
    // let mut guard = map.lock().unwrap();
    //
    // guard
    //     .entry(12)
    //     .or_insert_with(Vec::new)
    //     .push(pool);
    Ok(())
}
//
// /**
//  * 通过客户端ID获取一个连接
//  * @param clientID 客户端ID
//  */
// func get(clientID int) net.Conn {
//     poolLock.Lock()
//
//     //客户端连接池
//     poolList := poolMap[clientID]
//     if poolList == nil || len(*poolList) == 0 {
//         poolLock.Unlock()
//         return nil
//     }
//     var resultTcp net.Conn = nil
//     if len(*poolList) > 0 {
//
//         //取最后一次添加到连接池的连接
//         pool := (*poolList)[len(*poolList)-1]
//
//         //移除最后一个元素
//         *poolList = (*poolList)[:len(*poolList)-1]
//         //
//         ////取最后一次添加到连接池的连接
//         //pool := (*poolList)[0]
//         //
//         ////移除最后一个元素
//         //*poolList = (*poolList)[1:]
//
//         resultTcp = pool.PoolTCP
//         LogUtil.Debug(fmt.Sprintf("客户端ID:%d 当前连接池已经创建时间：%d秒\n", clientID, (time.Now().UnixMilli()-pool.CreateTime)/1000))
//     }
//     poolLock.Unlock()
//     return resultTcp
// }
//
// /**
//  * 从连接池获取一个连接,并请求添加连接池
//  * @param clientID 客户端ID
//  */
// func GetAndAddPool(clientID int) net.Conn {
//     var tcp net.Conn
//     for i := 0; i < 5; i++ {
//         tcp = get(clientID)
//         if tcp != nil {
//             break
//         }
//
//         //申请创建连接池
//         poolRequest(clientID, NPSConstant.ADD_POOL_COUNT)
//         time.Sleep(1 * time.Second)
//     }
//
//     //每消耗一个连接,申请创建两个连接,直到达到最大连接池数量
//     go poolRequest(clientID, 2)
//     return tcp
// }
//
// /**
//  * 发起连接池申请请求
//  * 每取走一个连接,则请求创建2个新的连接,直到达到最大连接数
//  * @param clientID 客户端ID
//  */
// func poolRequest(clientId int, count int) {
//     poolLock.Lock()
//     poolList := poolMap[clientId]
//     poolLock.Unlock()
//     if poolList == nil {
//         return
//     }
//     if len(*poolList) < NPSConstant.MAX_POOL_COUNT {
//         Csmi.SendTCPPoolRequest(clientId, count)
//     }
// }
//
// /**
//  * 移除某个客户端所有的连接池
//  * @param clientID 客户端ID
//  */
// func ShutdownByClient(clientID int) {
//     poolLock.Lock()
//     poolList := poolMap[clientID]
//     for _, it := range *poolList { //挨个关闭连接
//         it.PoolTCP.Close()
//     }
//     *poolList = []*TCPPool{}
//     poolLock.Unlock()
// }
//
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
