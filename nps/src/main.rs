// // #![allow(warnings)]//忽略所有警告
// mod application;
// mod constant;
mod dao;
// mod entity;
// mod extension;
// mod model;
// mod nps;
// mod util;
// mod web;

// use std::sync::LazyLock;
// use clap::Parser;
// use sqlx::sqlite::SqlitePoolOptions;
// use sqlx::SqlitePool;
// use dao::client_dao;

use crate::dao::channel_dao;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    // let arg = application::Args::try_parse().unwrap();
    // println!("-->ARG: {:?}", arg);
    // if arg.is_restart_mode {
    //     // 如果是重启模式，等待一段时间后再继续执行,以确保旧进程已经完全退出
    //     // tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    // }

    println!("-->START<--");
    // util::db_util::init().await;
    db::init().await;
    let id = channel_dao::insert(
        &db::DB_CONN.clone(),
        channel_dao::Channel {
            id: 0,
            client_id: 1,
            name: "test".to_string(),
            mode: 1,
            server_port: 8080,
            target_port: "80".to_string(),
            in_data: 0,
            out_data: 0,
            enable_state: 1,
            security_state: 1,
            acl_state: 1,
            created_at: 0,
            updated_at: 0,
            remark: None,
            error: None,
            version: 0,
        },
    )
    .await
    .unwrap();
    println!("插入数据id: {}", id);
    let channel = channel_dao::select_one(&db::DB_CONN.clone(), id)
        .await
        .unwrap();
    println!("查询结果: {:?}", channel);

    // util::security_util::init();
    // nps::init();
    // web::router::ready();
    // nps::nps_client::tcp_client::tcp_client_accept::accept().await?;
    //
    // //防止重启时，新进程还未完全启动，旧进程就已经退出了
    // tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    // println!("-->FINISH<--");
    Ok(())
}
