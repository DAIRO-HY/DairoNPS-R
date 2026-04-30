use crate::model::data_io_len::AtomicDataIOLen;
use crate::nps::TCPBridging;
use crate::nps::security_util::SERVER_SECURITY_KEY;
use crate::nps_error::NpsError;
use crate::{application, nps};
use np_common::{head_flag, time_util};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Notify;
use tokio::{io, select, try_join};

/// 桥接参数
pub struct TcpBridgeParam {
    pub ip: String,             // 代理客户端ip地址
    pub channel_id: i64,        // 隧道ID
    pub is_stats_traffic: bool, //是否实时统计流量
    pub target_port: String,    //目标端口
    pub security_state: i64,    //是否加密传输
    pub proxy_tcp: TcpStream,   //代码tcp
    pub client_tcp: TcpStream,  //客户端tcp
    pub data_len: AtomicDataIOLen, //流量统计
                                // pub closer: Arc<Notify>,                          //关闭监听器
                                // pub bridge_count: Arc<AtomicUsize>,               //统计桥接数
}

/// 准备开始桥接
pub async fn ready(param: TcpBridgeParam, bridge_count: Arc<AtomicUsize>, channel_closer: Arc<Notify>) {
    tokio::spawn(async move {
        let bridge_tag = application::BRIDGE_NEXT_TAG.fetch_add(1, Ordering::Relaxed);

        //桥接数+1
        bridge_count.fetch_add(1, Ordering::Relaxed);
        spawn_start(param, channel_closer, bridge_tag).await;

        //桥接数-1
        bridge_count.fetch_sub(1, Ordering::Relaxed);

        //桥接结束后,移除桥接信息,这里不能放到start函数中,因为start函数中可能会发生错误导致提前返回，这样就无法移除桥接信息了
        nps::CHANNEL_BRIDGING_MAP.remove(&bridge_tag);
    });
}

async fn spawn_start(param: TcpBridgeParam, channel_closer: Arc<Notify>, bridge_tag: u64) {
    select! {
        _ = channel_closer.notified() => {

            //这里无需显示关闭,生命周期结束之后会自动关闭并通知对方我方已经关闭
            // let _ = bridge_half.proxy_writer.shutdown().await;
            // let _ = bridge_half.client_writer.shutdown().await;
            println!("收到关闭通知，准备关闭桥接通信...");
            return;
        }
        result = start(param, bridge_tag) => {
            if let Err(e) = result {
                println!("桥接通信接发生了错误:{:?}", e);
            }
        }
    }
}

/**
 * 开始桥接传输数据
 */
async fn start(mut param: TcpBridgeParam, bridge_tag: u64) -> Result<(), NpsError> {

    //发送连接目标服务器标记
    param.client_tcp
        .write_u8(head_flag::CONNECT_TO_TARGET_SERVER)
        .await?;

    //将加密类型及目标端口 格式:加密状态|端口  1|80   1|127.0.0.1:80
    //1:加密  0:不加密
    let mut target_info = String::new();
    target_info.push_str(&(param.security_state.to_string()));
    target_info.push('|');
    target_info.push_str(param.target_port.as_str());
    param.client_tcp.write_u8(target_info.len() as u8).await?;
    param.client_tcp.write_all(target_info.as_bytes()).await?;

    if !param.is_stats_traffic {
        // 不需要实时统计流量
        return copy(param).await;
    }

    //统计当前桥接流量
    let data_len = AtomicDataIOLen::new();
    let bridge_closer = Arc::new(Notify::new());
    let last_rw_time = Arc::new(AtomicU64::new(time_util::current_millis()));

    //保存当前桥接信息，供监控使用
    nps::CHANNEL_BRIDGING_MAP.insert(
        bridge_tag,
        TCPBridging {
            ip: param.ip.clone(),
            channel_id: param.channel_id,
            data_len: data_len.clone(),
            create_time: time_util::current_millis(),
            last_rw_time: last_rw_time.clone(),
            closer: bridge_closer.clone(),
        },
    );
    let (proxy_reader, proxy_writer) = tokio::io::split(param.proxy_tcp);
    let (client_reader, client_writer) = tokio::io::split(param.client_tcp);

    let p2c = proxy_to_client(
        param.security_state == 1,
        data_len.clone(),
        param.data_len.clone(),
        proxy_reader,
        client_writer,
        last_rw_time.clone(),
    );
    let c2p = client_to_proxy(
        param.security_state == 1,
        data_len,
        param.data_len.clone(),
        client_reader,
        proxy_writer,
        last_rw_time,
    );

    select! {
        _ = bridge_closer.notified() => {
            //这里无需显示关闭,生命周期结束之后会自动关闭并通知对方我方已经关闭
            // let _ = param.proxy_writer.shutdown().await;
            // let _ = param.client_writer.shutdown().await;
            // println!("收到关闭桥接通知，准备关闭桥接通信...");
            Ok(())
        }
        result = async{try_join!(p2c, c2p)} => {
            result?;
            Ok(())
        }
    }
}

/// 不需要实时统计流量，高性能模式
async fn copy(
    mut param: TcpBridgeParam
) -> Result<(), NpsError> {
    let (in_len, out_len) = io::copy_bidirectional(&mut param.client_tcp, &mut param.proxy_tcp).await?;
            param.data_len.add_in(in_len);
            param.data_len.add_out(out_len);
            // println!("-->in_len:{}", in_len);
            // println!("-->out_len:{}", out_len);
            Ok(())
}

async fn proxy_to_client(
    need_encryption: bool,
    bridge_data_len: AtomicDataIOLen,
    channel_data_len: AtomicDataIOLen,
    mut proxy_reader: ReadHalf<TcpStream>,
    mut client_writer: WriteHalf<TcpStream>,
    last_rw_time: Arc<AtomicU64>,
) -> io::Result<()> {
    let mut buf = [0u8; 1024 * 8];

    //使用&*,避免发生值复制
    let security_keys = &*SERVER_SECURITY_KEY;

    loop {
        let n = proxy_reader.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        //记录最后一次读写时间
        last_rw_time.store(time_util::current_millis(), Ordering::Relaxed);
        bridge_data_len.add_in(n);
        channel_data_len.add_in(n);
        if need_encryption {
            //需要加密处理
            for b in &mut buf[..n] {
                *b = security_keys[*b as usize];
            }
        }
        client_writer.write_all(&buf[..n]).await?;
    }

    //这里必须关闭客户端的输出流，否则对方无法感知到已经关闭连接了（写失败或者读失败没有必要调用shutdown()，即使调用大概率也是失败的，所以没有意义）
    client_writer.shutdown().await?;
    // println!("-->客户端连接已关闭");
    Ok(())
}

async fn client_to_proxy(
    need_encryption: bool,
    bridge_data_len: AtomicDataIOLen,
    channel_data_len: AtomicDataIOLen,
    mut client_reader: ReadHalf<TcpStream>,
    mut proxy_writer: WriteHalf<TcpStream>,
    last_rw_time: Arc<AtomicU64>,
) -> io::Result<()> {
    let mut buf = [0u8; 1024 * 8];

    //使用&*,避免发生值复制
    let security_keys = &*SERVER_SECURITY_KEY;
    loop {
        let n = client_reader.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        //记录最后一次读写时间
        last_rw_time.store(time_util::current_millis(), Ordering::Relaxed);
        bridge_data_len.add_out(n);
        channel_data_len.add_out(n);
        if need_encryption {
            //需要解密处理
            for b in &mut buf[..n] {
                *b = security_keys[*b as usize];
            }
        }
        proxy_writer.write_all(&buf[..n]).await?;
    }
    //这里必须关闭客户端的输出流，否则对方无法感知到已经关闭连接了（写失败或者读失败没有必要调用shutdown()，即使调用大概率也是失败的，所以没有意义）
    proxy_writer.shutdown().await?;
    // println!("-->代理连接已关闭");
    Ok(())
}
