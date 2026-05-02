#![allow(warnings)]

//忽略所有警告
mod tcp_bridge;
mod npc_error;
mod tcp_pool;
mod session;
mod application;
mod security_util;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    session::open().await;
    println!("-->程序已退出");
    Ok(())
}
