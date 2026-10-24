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

/// 开启服务,会阻塞直到服务停止,在命令行程序中调用
pub fn start(args: Option<Argument>) {
    let args = args.unwrap_or_else(Argument::parse);
    if !session::try_open() {
        println!("-->NPC服务已经在运行中");
        return;
    }
    run(args);
}

/// 在新线程中开启服务,在iOS或Android中调用
/// 函数返回前就已经标记为开启状态,所以紧接着调用stop()也能正确停止服务
pub fn start_in_thread(args: Argument) {
    if !session::try_open() {
        //已经在运行中
        return;
    }
    std::thread::spawn(move || run(args));
}

/// 运行服务,直到服务停止
fn run(args: Argument) {
    println!("args: {:?}", args);
    RUNTIME.block_on(async {
        session::open(args).await;
        *application::NPC_CONNECT_MSG.lock().unwrap() = "NPC服务已关闭".to_string();
    });
    println!("-->程序已退出");
}

/// 停止服务
pub fn stop() {
    RUNTIME.block_on(async {
        session::stop().await;
    });
}
