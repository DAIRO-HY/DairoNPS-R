#![allow(warnings)] //忽略所有警告
mod application;
mod constant;
mod dao;
mod entity;
mod extension;
mod forward;
mod model;
mod nps;
mod nps_error;
mod util;
mod web;

use itertools::Itertools;
use sqlx::{Column, Executor, Statement, TypeInfo};

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
    web::router::ready();

    //开启内网穿透监听
    nps::ready();

    //开启端口转发监听
    forward::read();

    //等待程序推出
    application::SHUTDOWN_NOTIFY.notified().await;
    println!("-->即将退出");
    
    // 等待一段时间再退出确保监听端口都已经关闭
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    println!("-->FINISH<--");
    Ok(())
}
