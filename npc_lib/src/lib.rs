#![allow(warnings)]

use crate::application::Argument;
use std::sync::{LazyLock, OnceLock};
use clap::Parser;
use tokio::runtime::{Builder, Runtime};

//忽略所有警告
pub mod application;
mod npc_error;
mod security_util;
pub mod session;
mod tcp_bridge;
mod tcp_pool;

pub static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
    // Builder::new_current_thread()
    Builder::new_multi_thread().enable_all().build().unwrap()
});

/// 开启服务,在iOS或Android中调用
pub fn start(args: Option<Argument>) {
    let args = args.unwrap_or(Argument::try_parse().unwrap());
    RUNTIME.block_on(async {
        session::open(args).await;
        *application::NPC_CONNECT_MSG.write().await = "NPC服务已关闭".to_string();
    });
    println!("-->程序已退出");
}

/// 停止服务
pub fn stop() {
    RUNTIME.block_on(async {
        session::stop().await;
    });
}
