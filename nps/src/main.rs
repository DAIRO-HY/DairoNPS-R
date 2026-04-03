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

// use std::sync::LazyLock;
// use clap::Parser;
// use sqlx::sqlite::SqlitePoolOptions;
// use sqlx::SqlitePool;
// use dao::client_dao;

use crate::dao::channel_dao;
use sqlx::{Column, Executor, Statement, TypeInfo};

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    // let arg = application::Args::try_parse().unwrap();
    // println!("-->ARG: {:?}", arg);
    // if arg.is_restart_mode {
    //     // 如果是重启模式，等待一段时间后再继续执行,以确保旧进程已经完全退出
    //     // tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    // }

    tokio::spawn(async {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            println!(
                "-->client_session_count:{}",
                nps::CLIENT_SESSION.lock().await.len()
            );
            let pool_map = nps::POOL_MAP.lock().await;
            pool_map
                .iter()
                .for_each(|it| println!("-->client:{} pool_count:{}", it.0, it.1.len()));
        }
    });

    println!("-->START<--");
    db::init().await;

    // let id = channel_dao::insert(
    //     &db::get(),
    //     channel_dao::Channel {
    //         id: 0,
    //         client_id: 1,
    //         name: "test".to_string(),
    //         mode: 1,
    //         server_port: 8080,
    //         target_port: "80".to_string(),
    //         in_data: 0,
    //         out_data: 0,
    //         enable_state: 1,
    //         security_state: 1,
    //         acl_state: 1,
    //         ..Default::default()
    //     },
    // )
    // .await
    // .unwrap();
    // println!("插入数据id: {}", id);
    // let channel = channel_dao::select_one(&db::get(), id)
    //     .await
    //     .unwrap();
    // println!("查询结果: {:?}", channel);
    //
    // let count = channel_dao::update(&db::get(),channel.clone()).await.unwrap();
    // println!("更新结果: {}", count);
    //
    // let all = channel_dao::select_all(&db::get())
    //     .await
    //     .unwrap();
    // println!("查询结果: {:?}", all);
    //
    // let count = channel_dao::set_delete(&db::get(),id, channel.version + 1).await.unwrap();
    // println!("删除结果: {}", count);
    //
    // let count = channel_dao::set_delete_ignone_version(&db::get(),id).await.unwrap();
    // println!("删除结果: {}", count);
    //
    // let count = channel_dao::delete(&db::get(),id).await.unwrap();
    // println!("删除结果: {}", count);

    util::security_util::init();
    nps::init();
    web::router::ready();
    nps::nps_client::tcp_client::tcp_client_accept::accept().await?;

    //防止重启时，新进程还未完全启动，旧进程就已经退出了
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    println!("-->FINISH<--");
    Ok(())
}
