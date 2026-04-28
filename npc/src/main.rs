#![allow(warnings)]

use std::sync::atomic::Ordering;
use std::time::Duration;
use tokio::time::sleep;

//忽略所有警告
mod bridge;
mod npc_error;
mod pool;
mod session;
// mod header_util;
mod application;
mod security_util;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    println!("hello npc");
    tokio::spawn(async {
        loop {
            sleep(Duration::from_millis(1000)).await;
            println!(
                "当前连接池数:{} 桥接数:{} ",
                application::POOL_COUNT.load(Ordering::Relaxed),
                application::BRIDGE_COUNT.load(Ordering::Relaxed),
            );
        }
    });
    // tokio::spawn(async{
    //     sleep(Duration::from_millis(10000)).await;
    //     session::npc_session::close().await;
    //     println!("closed");
    // });
    // tokio::spawn(async{
    //     sleep(Duration::from_millis(20000)).await;
    //     println!("restart");
    //     session::npc_session::open().await;
    // });
    session::npc_session::open().await;
    println!("--->FINISH");
    sleep(Duration::from_millis(100000000)).await;
    Ok(())
}
