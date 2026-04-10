#![allow(warnings)] //忽略所有警告
mod application;
mod constant;
mod dao;
mod entity;
mod extension;
mod model;
mod nps;
mod util;
mod web;

use crate::model::data_io_len::AtomicDataIOLen;
use itertools::Itertools;
use sqlx::{Column, Executor, Statement, TypeInfo};
use tokio::{io, try_join};
use crate::nps::nps_error::NpsError;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    // let arg = application::Args::try_parse().unwrap();
    // println!("-->ARG: {:?}", arg);
    // if arg.is_restart_mode {
    //     // 如果是重启模式，等待一段时间后再继续执行,以确保旧进程已经完全退出
    //     // tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    // }
    println!("-->START<--");
    db::init().await;
    nps::init();
    web::router::ready();
    if let Err(e) = nps::nps_client::tcp_client::tcp_client_accept::accept().await{
        println!("监听客户端发生了错误:{:?}", e);
    }

    //防止重启时，新进程还未完全启动，旧进程就已经退出了
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    println!("-->FINISH<--");
    Ok(())
}
