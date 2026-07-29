#![allow(warnings)]

//忽略所有警告
mod application;
mod code;
mod dao;
mod entity;
mod extension;
mod forward;
mod model;
mod nps;
mod nps_error;
mod util;
mod web;

use mimalloc::MiMalloc;
use std::str::FromStr;

// 接管内存分配,比系统分配收益更高
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    application::init();
    lib_db::init("data/dairo-nps.sqlite").await;
    web::router::ready();

    //等待程序推出
    application::SHUTDOWN_NOTIFY.notified().await;
    println!("-->即将退出");

    // 等待一段时间再退出确保监听端口都已经关闭
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    println!("-->程序已退出");
    Ok(())
}

// 用来把不使用的内存页归还给OS
unsafe extern "C" {
    pub fn mi_collect(force: bool);
}
