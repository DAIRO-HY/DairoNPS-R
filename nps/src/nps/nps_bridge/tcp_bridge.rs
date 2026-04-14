use crate::model::data_io_len::AtomicDataIOLen;
use crate::{application, nps};
use crate::nps::security_util::SERVER_SECURITY_KEY;
use crate::nps::TCPBridging;
use crate::nps_error::NpsError;
use crate::util::time_util;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Notify;
use tokio::{io, select, try_join};

/// 桥接参数
pub struct TcpBridgeParam {
    pub ip: String,                                   // 代理客户端ip地址
    pub channel_id: i64,                              // 隧道ID
    pub is_stats_traffic: bool,                       //是否实时统计流量
    pub target_port: String,                          //目标端口
    pub security_state: i64,                          //是否加密传输
    pub proxy_tcp: TcpStream,                         //代码tcp
    pub client_tcp: TcpStream,                        //客户端tcp
    pub data_len: AtomicDataIOLen,                    //流量统计
    pub closer: Arc<Notify>,                          //关闭监听器
    pub bridge_count: Arc<AtomicUsize>,               //统计桥接数
}

/// 准备开始桥接
pub async fn ready(param: TcpBridgeParam) {
    tokio::spawn(async move {
        if let Err(e) = start(param).await {
            println!("桥接通信接发生了错误:{:?}", e);
        }
    });
}

/**
 * 开始桥接传输数据
 */
async fn start(param: TcpBridgeParam) -> Result<(), NpsError> {
    //桥接数+1
    param.bridge_count.fetch_add(1, Ordering::Relaxed);

    let (proxy_reader, proxy_writer) = tokio::io::split(param.proxy_tcp);
    let (client_reader, mut client_writer) = tokio::io::split(param.client_tcp);

    //将加密类型及目标端口 格式:加密状态|端口  1|80   1|127.0.0.1:80
    //1:加密  0:不加密
    let mut header = String::new();
    header.push_str(&(param.security_state.to_string()));
    header.push('|');
    header.push_str(param.target_port.as_str());

    //发送目标端口信息
    if let Err(e) = send_header_to_client(header, &mut client_writer).await {
        //桥接数-1
        param.bridge_count.fetch_sub(1, Ordering::Relaxed);
        return Err(e);
    }
    if !param.is_stats_traffic {
        // 不需要实时统计流量
        let result = copy(
            proxy_reader,
            proxy_writer,
            client_writer,
            client_reader,
            param.closer,
            param.data_len,
        )
        .await;

        //桥接数-1
        param.bridge_count.fetch_sub(1, Ordering::Relaxed);
        return result;
    }

    //统计当前桥接流量
    let data_len = AtomicDataIOLen::new();
    let tag = application::BRIDGE_NEXT_TAG.fetch_add(1, Ordering::Relaxed);
    let bridge_closer = Arc::new(Notify::new());
    let last_rw_time = Arc::new(AtomicU64::new(time_util::current_millis()));

    //保存当前桥接信息，供监控使用
    nps::CHANNEL_BRIDGING_MAP.insert(
        tag,
        TCPBridging {
            ip: param.ip,
            channel_id: param.channel_id,
            data_len: data_len.clone(),
            create_time: time_util::current_millis(),
            last_rw_time: last_rw_time.clone(),
            closer:bridge_closer.clone()
        },
    );

    let p2c = proxy_to_client(
        param.security_state == 1,
        data_len.clone(),
        param.data_len.clone(),
        proxy_reader,
        client_writer,
        param.closer.clone(),
        bridge_closer.clone(),
        last_rw_time.clone(),
    );
    let c2p = client_to_proxy(
        param.security_state == 1,
        data_len,
        param.data_len,
        client_reader,
        proxy_writer,
        param.closer,
        bridge_closer,
        last_rw_time,
    );
    let result = try_join!(p2c, c2p);

    //桥接结束后,移除桥接信息
    nps::CHANNEL_BRIDGING_MAP.remove(&tag);

    //桥接数-1
    param.bridge_count.fetch_sub(1, Ordering::Relaxed);
    result?;
    Ok(())
}

/// 不需要实时统计流量，高性能模式
async fn copy(
    mut proxy_reader: ReadHalf<TcpStream>,
    mut proxy_writer: WriteHalf<TcpStream>,
    mut client_writer: WriteHalf<TcpStream>,
    mut client_reader: ReadHalf<TcpStream>,
    closer: Arc<Notify>,
    data_len: AtomicDataIOLen,
) -> Result<(), NpsError> {
    select! {
        _ = closer.notified() => {
            proxy_writer.shutdown().await?;
            client_writer.shutdown().await?;
            return Ok(());
        }
        rs = async {
            try_join!(
                async{
                    let len = io::copy(&mut proxy_reader, &mut client_writer).await;
                    client_writer.shutdown().await?;
                    len
                },
                async{
                    let len = io::copy(&mut client_reader, &mut proxy_writer).await;
                    proxy_writer.shutdown().await?;
                    len
                }
            )
        } => {
            return match rs{
                Ok((in_len,out_len))=>{
                    data_len.add_in(in_len);
                    data_len.add_out(out_len);
                    println!("-->in_len:{}",in_len);
                    println!("-->out_len:{}",out_len);
                    Ok(())
                },
                Err(e) =>{
                    println!("-->e:{:?}",e);
                    Err(NpsError::IoError(e))
                }
            }
        }
    }
}

async fn proxy_to_client(
    need_encryption: bool,
    bridge_data_len: AtomicDataIOLen,
    channel_data_len: AtomicDataIOLen,
    mut proxy_reader: ReadHalf<TcpStream>,
    mut client_writer: WriteHalf<TcpStream>,
    closer: Arc<Notify>,
    bridge_closer: Arc<Notify>,
    last_rw_time:Arc<AtomicU64>
) -> io::Result<()> {
    let mut buf = [0u8; 1024 * 8];
    loop {
        select! {
            _ = closer.notified() => {
                break;
            }
            _ = bridge_closer.notified() => {
                break;
            }
            read_data = proxy_reader.read(&mut buf) => {
                let n = read_data?;
                if n == 0 {
                    break;
                }

                //记录最后一次读写时间
                last_rw_time.store(time_util::current_millis(),Ordering::Relaxed);
                bridge_data_len.add_in(n);
                channel_data_len.add_in(n);
                if need_encryption{//需要加密处理
                    for b in &mut buf[..n] {
                        *b = SERVER_SECURITY_KEY[*b as usize];
                    }
                }
                client_writer.write_all(&buf[..n]).await?;
            }
        }
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
    closer: Arc<Notify>,
    bridge_closer: Arc<Notify>,
    last_rw_time:Arc<AtomicU64>,
) -> io::Result<()> {
    let mut buf = [0u8; 1024 * 8];
    loop {
        select! {
            _ = closer.notified() => {
                break;
            }
            _ = bridge_closer.notified() => {
                break;
            }
            read_data = client_reader.read(&mut buf) =>{
                let n = read_data?;
                if n == 0 {
                    break;
                }

                //记录最后一次读写时间
                last_rw_time.store(time_util::current_millis(),Ordering::Relaxed);
                bridge_data_len.add_out(n);
                channel_data_len.add_out(n);
                if need_encryption {
                    //需要解密处理
                    for b in &mut buf[..n] {
                        *b = SERVER_SECURITY_KEY[*b as usize];
                    }
                }
                proxy_writer.write_all(&buf[..n]).await?;
            }
        }
    }
    //这里必须关闭客户端的输出流，否则对方无法感知到已经关闭连接了（写失败或者读失败没有必要调用shutdown()，即使调用大概率也是失败的，所以没有意义）
    proxy_writer.shutdown().await?;
    // println!("-->代理连接已关闭");
    Ok(())
}

/**
 * 发送目标端口信息
 */
async fn send_header_to_client(
    header: String,
    client_writer: &mut WriteHalf<TcpStream>,
) -> Result<(), NpsError> {
    let mut data = Vec::with_capacity(header.len() + 1);
    data.push(header.len() as u8); //写入数据长度标识
    data.extend_from_slice(header.as_bytes()); //写入数据

    client_writer.write_all(&data).await?;
    Ok(())
}
