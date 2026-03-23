use std::collections::HashMap;

use crate::nps::nps_bridge::tcp::tcp_bridge::TCPBridge;
use std::sync::LazyLock;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::try_join;

static NPS_SERVER: LazyLock<Mutex<HashMap<String, TcpStream>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

async fn ready() {
    let mut socket_map = NPS_SERVER.lock().await;
    if socket_map.contains_key("nps_server") && socket_map.contains_key("proxy_server") {
        println!("NPS服务器和代理服务器均已就绪，可以开始数据传输了");

        //从HashMap中拿走所有权
        let nps_stream = socket_map.remove("nps_server").unwrap();
        let proxy_stream = socket_map.remove("proxy_server").unwrap();
        drop(socket_map); //释放锁

        let (proxy_reader, proxy_writer) = tokio::io::split(proxy_stream);
        let (client_reader, client_writer) = tokio::io::split(nps_stream);

        let bridge = TCPBridge {
            proxy_reader,
            proxy_writer,
            client_reader,
            client_writer,
        };
        bridge.start().await.unwrap();
    }
}

async fn nps_server_accept() {
    let listener = TcpListener::bind("0.0.0.0:1881").await.unwrap();
    let (stream, _) = listener.accept().await.unwrap();
    NPS_SERVER
        .lock()
        .await
        .insert("nps_server".to_string(), stream);
    ready().await;
}

async fn proxy_server_accept() {
    let listener = TcpListener::bind("0.0.0.0:1882").await.unwrap();
    let (stream, _) = listener.accept().await.unwrap();
    NPS_SERVER
        .lock()
        .await
        .insert("proxy_server".to_string(), stream);
    ready().await;
}

async fn nps_client() -> tokio::io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:1881").await.unwrap();
    let (mut reader, mut writer) = tokio::io::split(stream);

    let read_fn = async move {
        let mut buf = [0u8; 4096];
        for _ in 0.. {
            let n = reader.read(&mut buf).await.unwrap();
            if n == 0 {
                break;
            }
            println!("nps_client收到数据:{}", String::from_utf8_lossy(&buf[..n]));
        }
        Ok::<(), tokio::io::Error>(())
    };
    let write_fn = async move {
        for i in 0..10 {
            writer
                .write_all(format!("-->nps_client:{}", i).as_bytes())
                .await
                .unwrap();
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        writer.shutdown().await?;
        Ok::<(), tokio::io::Error>(())
    };
    try_join!(read_fn, write_fn)?;
    println!("------->nps_client数据传输结束");
    Ok(())
}

async fn proxy_client() -> tokio::io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:1882").await.unwrap();
    let (mut reader, mut writer) = tokio::io::split(stream);

    let read_fn = async move {
        let mut buf = [0u8; 4096];
        for _ in 0.. {
            let n = reader.read(&mut buf).await.unwrap();
            if n == 0 {
                break;
            }
            println!(
                "proxy_client收到数据:{}",
                String::from_utf8_lossy(&buf[..n])
            );
        }
        Ok::<(), tokio::io::Error>(())
    };
    let write_fn = async move {
        for i in 0..5 {
            writer
                .write_all(format!("-->proxy:{}", i).as_bytes())
                .await
                .unwrap();
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        writer.shutdown().await?;
        Ok::<(), tokio::io::Error>(())
    };
    try_join!(read_fn, write_fn)?;
    println!("------->proxy_client数据传输结束");
    Ok(())
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    #[ignore]
    async fn bridge_test() -> tokio::io::Result<()> {
        println!("------------------------------------>测试开始");
        tokio::join!(
            tokio::spawn(super::nps_server_accept()),
            tokio::spawn(super::proxy_server_accept()),
            tokio::spawn(super::nps_client()),
            tokio::spawn(super::proxy_client())
        );
        println!("------------------------------------>测试结束");
        Ok(())
    }
}
