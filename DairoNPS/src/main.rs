// #![allow(warnings)] //忽略所有警告
mod application;
mod dao;
mod entity;
mod extension;
mod forward;
mod model;
mod nps;
mod nps_error;
mod util;
mod web;

use crate::extension::number::NumberExtension;
use crate::nps::nps_client;
use itertools::Itertools;
use sqlx::{Column, Executor, Statement, TypeInfo};

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    println!("{:?}", *application::ARGS);

    //数据保存目录不存在时创建
    if let Err(e) = std::fs::create_dir_all(&application::ARGS.data_dir) {
        eprintln!("-->创建数据保存目录:{}失败: {}", application::ARGS.data_dir, e);
        std::process::exit(1);
    }
    lib_db::init(&application::ARGS.data_dir).await;
    web::server::ready();

    //开启内网穿透监听
    nps_client::ready();

    //开启端口转发监听
    forward::read();

    //等待程序推出
    application::SHUTDOWN_NOTIFY.notified().await;
    println!("-->即将退出");

    // 等待一段时间再退出确保监听端口都已经关闭
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    println!("-->程序已退出");
    Ok(())
}
