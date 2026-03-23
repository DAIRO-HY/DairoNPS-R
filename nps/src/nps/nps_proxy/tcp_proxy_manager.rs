use crate::dao::channel_dao;
use crate::entity::channel::Channel;
use crate::model::data_total::DataTotal;
use crate::nps;
use crate::nps::nps_proxy::tcp_proxy_accept::TCPProxyAccept;
use crate::nps::{CHANNEL_CLOSE_NOTIFY, CHANNEL_DATA_TOTAL};
use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::{net::TcpListener, sync::Notify};
// // 隧道id对应的服务端口监听
// var proxyAcceptMap = make(map[int]*TCPProxyAccept)

// // proxyAcceptMap操作互斥锁
// var proxyAcceptLock sync.Mutex

// // 隧道代理端口数量
// func GetProxyCount() int {
// 	count := 0
// 	proxyAcceptLock.Lock()
// 	count = len(proxyAcceptMap)
// 	proxyAcceptLock.Unlock()
// 	return count
// }

pub fn init() {
    tokio::spawn(channel_data_stats());
}

// 开始客户端的所有监听
pub async fn accept_client(client_id: i64) {
    // //加载统计数据
    // ChannelStatisticsUtil.Init()

    //开启NPS客户端ID下所有的隧道
    let active_list = channel_dao::select_active_by_client_id(client_id);
    for it in active_list {
        if it.mode == 1 {
            //只监听TCP隧道
            accept_channel(it).await;
        }
    }
}

// 开始监听某个隧道
async fn accept_channel(channel: Channel) {
    // proxyAcceptLock.Lock()
    // oldProxyTCPAccept := proxyAcceptMap[channel.Id]
    // if oldProxyTCPAccept != nil { //若该隧道已经在监听,则先停止
    // 	shutdown(oldProxyTCPAccept)
    // }

    //关闭隧道正在通信的连接
    shutdown_by_channel(channel.id).await;
    println!(
        "-->开始监听隧道: {} 代理端口: {} 目标端口: {}",
        channel.id, channel.server_port, channel.target_port
    );
    let tcp_listener = TcpListener::bind(format!("0.0.0.0:{}", channel.server_port))
        .await
        .unwrap();
    // listener, err := net.Listen("tcp", ":"+strconv.Itoa(channel.ServerPort))
    // if err != nil {
    // 	errMsg := fmt.Sprintf("端口:%d 监听失败。err:%q\n", channel.ServerPort, err)
    // 	ChannelDao.SetError(channel.Id, &errMsg)
    // 	LogUtil.Error(errMsg)
    // 	proxyAcceptLock.Unlock()
    // 	return
    // }
    // ChannelDao.SetError(channel.Id, nil)
    // LogUtil.Info(fmt.Sprintf("端口:%d 监听开始\n", channel.ServerPort))
    // proxyAccept := &TCPProxyAccept{
    // 	ClientId: ClientId,
    // 	Channel:  channel,
    // 	listen:   listener,
    // }
    // proxyAcceptMap[channel.Id] = proxyAccept
    // proxyAcceptLock.Unlock()

    let channel_id = channel.id;
    let data_total = DataTotal::from(channel.in_data, channel.out_data);
    let notify = Arc::new(Notify::new());
    let proxy_tcp_accept = TCPProxyAccept {
        channel,
        tcp_listener,
        notify: notify.clone(),
        data_total: data_total.clone(),
    };

    //保存关闭通知器
    CHANNEL_CLOSE_NOTIFY
        .get()
        .unwrap()
        .lock()
        .await
        .insert(channel_id, notify.clone());

    //初始化隧道数据总量
    CHANNEL_DATA_TOTAL
        .get()
        .unwrap()
        .lock()
        .await
        .insert(channel_id, data_total);
    tokio::spawn(async move {
        let _ = proxy_tcp_accept.accept().await;

        //接受到accept结束的通知,说明监听已经停止,可以安全地删除关闭通知器
        CHANNEL_CLOSE_NOTIFY
            .get()
            .unwrap()
            .lock()
            .await
            .remove(&channel_id);
    });
}

// 关闭监听
// - channelId 隧道id
async fn shutdown_by_channel(channel_id: i64) {
    // proxyAcceptLock.Lock()
    // proxyTCPAccept := proxyAcceptMap[channelId]
    // if proxyTCPAccept != nil {
    // 	shutdown(proxyTCPAccept)
    // }
    // proxyAcceptLock.Unlock()

    loop {
        //等待隧道代理监听停止,否则可能导致下次监听同一端口失败
        if let Some(notify) = CHANNEL_CLOSE_NOTIFY
            .get()
            .unwrap()
            .lock()
            .await
            .get(&channel_id)
        {
            println!(
                "-->等待隧道代理监听停止,否则可能导致下次监听同一端口失败。channelId: {}",
                channel_id
            );
            let _ = notify.notify_one();
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        } else {
            return;
        }
    }
    // ss.map(|notify| {
    // 	notify.notify_one();
    // });

    //关闭隧道所有正在通信的连接
    // tcp_bridge.ShutdownByChannel(channelId)
}

// // 关闭某个客户端下所有的隧道
// func ShutdownByClient(clientId int) {

// 	//关闭客户端所有隧道
// 	channelIdList := ChannelDao.SelectIdByClientId(clientId)
// 	for _, it := range channelIdList {
// 		ShutdownByChannel(it)
// 	}

// 	//关闭客户端所有正在通信的连接
// 	tcp_bridge.ShutdownByClient(clientId)
// }

// // 停止监听端口
// func shutdown(proxyTCPAccept *TCPProxyAccept) {
// 	proxyTCPAccept.listen.Close()
// 	channelId := proxyTCPAccept.Channel.Id
// 	if proxyAcceptMap[channelId] != nil {
// 		delete(proxyAcceptMap, channelId)
// 	}
// }

// 统计隧道数据总量
async fn channel_data_stats() {
    const STATS_INTERVAL: u64 = 1000; //统计间隔，单位毫秒
    let conn = crate::util::db_util::new_connection();

    //最后一次统计到的隧道数据总量
    let mut last_total_map: HashMap<i64, DataTotal> = HashMap::new();
    if DataTotal::from(0, 0) == DataTotal::from(0, 0) {
        println!("DataTotal PartialEq implemented correctly");
    } else {
        println!("DataTotal PartialEq implementation error");
    }
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(STATS_INTERVAL)).await;
        let data_map = CHANNEL_DATA_TOTAL.get().unwrap().lock().await;
        for (channel_id, data_total) in data_map.iter() {
            if let Some(last_total) = last_total_map.get(channel_id)
                && last_total == data_total
            {
                //如果数据总量没有变化，说明隧道没有流量了，可以删除统计数据以节省内存
                break;
            }

            //每隔STATS_INTERVAL毫秒统计一次数据总量
            let in_data = data_total.load_in() as i64;
            let out_data = data_total.load_out() as i64;

            //这里可以将统计数据保存到数据库或者发送到监控系统
            crate::dao::channel_data_dao::add(&conn, channel_id.clone(), in_data, out_data);
            last_total_map.insert(*channel_id, DataTotal::from(in_data as u64, out_data as u64));
        }
    }
}
