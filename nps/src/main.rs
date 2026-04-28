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

use std::sync::{Arc, LazyLock};
use crate::extension::number::NumberExtension;
use np_common::time_util;
use itertools::Itertools;
use sqlx::{Column, Executor, Statement, TypeInfo};
use std::sync::atomic::{AtomicU64, Ordering};


static last_rw_time:LazyLock<AtomicU64> = LazyLock::new(||AtomicU64::new(0));

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    // let arg = application::Args::try_parse().unwrap();
    // println!("-->ARG: {:?}", arg);
    // if arg.is_restart_mode {
    //     // 如果是重启模式，等待一段时间后再继续执行,以确保旧进程已经完全退出
    //     // tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    // }


    // let now = time_util::current_millis();
    // for i in 0..100000000 {
    //     let now = time_util::current_millis();
    //     last_rw_time.store(now, Ordering::Relaxed);
    //     // if now == 10010{
    //     //     println!("wdssdds:{}",now)
    //     // }
    //     // if i == 100000001{
    //     //     println!("wdssdds:{}",i)
    //     // }
    // }
    // println!(
    //     "-->t:{}    {}",
    //     time_util::current_millis() - now,
    //     last_rw_time.load(Ordering::Relaxed)
    // );

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
