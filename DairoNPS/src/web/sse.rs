use crate::application;
use bytes::Bytes;
use hyperoute_core::Stream;
use std::future::Future;
use std::time::Duration;
use tokio::select;
use tokio_stream::wrappers::ReceiverStream;

/// 创建一个SSE响应，每隔一段时间调用一次f，将返回的数据推送给客户端
/// 客户端断开连接或程序退出时停止推送
pub fn interval<F, Fut>(interval: Duration, mut f: F) -> Stream
where
    F: FnMut() -> Fut + Send + 'static,
    Fut: Future<Output = String> + Send,
{
    let (tx, rx) = tokio::sync::mpsc::channel::<std::io::Result<Bytes>>(1);
    tokio::spawn(async move {
        loop {
            let event = Bytes::from(format!("data: {}\n\n", f().await));
            select! {
                _ = application::SHUTDOWN_NOTIFY.notified() => break,
                result = tx.send(Ok(event)) => {
                    if result.is_err() {
                        //客户端已经断开连接
                        break;
                    }
                }
            }
            select! {
                _ = application::SHUTDOWN_NOTIFY.notified() => break,
                _ = tx.closed() => break,
                _ = tokio::time::sleep(interval) => {}
            }
        }
    });
    Stream::new(ReceiverStream::new(rx))
        .header("content-type", "text/event-stream")
        .header("cache-control", "no-cache")
}
