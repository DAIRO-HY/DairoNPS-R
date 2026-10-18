use crate::application;
use std::sync::atomic::Ordering;
use tokio::net::TcpListener;
use tokio::select;

/// 由build.rs扫描注解生成的路由
mod routes {
    include!(concat!(env!("OUT_DIR"), "/routes.rs"));
}

pub fn ready() {
    tokio::spawn(start());
}

async fn start() {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", application::ARGS.web_port))
        .await
        .unwrap();
    loop {
        select! {
            _ = application::SHUTDOWN_NOTIFY.notified() => break,
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => {
                    tokio::spawn(hyperoute_core::serve_connection(stream, routes::router));
                }
                Err(e) => eprintln!("Web服务接收连接失败:{:?}", e),
            }
        }
    }

    //关闭监听,释放端口
    drop(listener);

    // 标记Web服务已经退出
    application::IS_WEB_DROP.store(true, Ordering::Relaxed);
}
