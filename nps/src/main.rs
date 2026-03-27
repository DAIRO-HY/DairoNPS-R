// #![allow(warnings)]//忽略所有警告
mod application;
mod constant;
mod dao;
mod entity;
mod extension;
mod model;
mod nps;
mod util;
mod web;

use clap::Parser;
use dao::client_dao;

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let arg = application::Args::try_parse().unwrap();
    println!("-->ARG: {:?}", arg);
    if arg.is_restart_mode {
        // 如果是重启模式，等待一段时间后再继续执行,以确保旧进程已经完全退出
        // tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    }
    println!("-->START<--");
    util::db_util::init().await;
    util::security_util::init();
    nps::init();
    web::router::ready();
    nps::nps_client::tcp_client::tcp_client_accept::accept().await?;

    // let conn = crate::util::db_util::new_connection();
    // let entity = client_dao::Client {
    //     name: "Client1".to_string(),
    //     key: chrono::Local::now().to_rfc3339(),
    //     ..Default::default()
    // };
    // match client_dao::insert(&conn, entity) {
    //     Ok(id) => println!("-->Client inserted successfully with id: {}", id),
    //     Err(err) => eprintln!("-->Error inserting client: {}", err),
    // }

    // if let Some(err) = client_dao::update(
    //     &conn,
    //     client_dao::Client {
    //         id: 1,
    //         name: "Client1".to_string(),
    //         key: chrono::Local::now().to_rfc3339(),
    //         version: 0,
    //         created_at: 0,
    //         deleted: 0,
    //         ..Default::default()
    //     },
    // ) {
    //     eprintln!("-->Error updating client: {}", err);
    // } else {
    //     println!("-->Client updated successfully");
    // }

    // if let Some(err) = client_dao::delete_ignore_version(&conn, 1, "admin".to_string()) {
    //     eprintln!("-->Error delete_ignore_version client: {}", err);
    // } else {
    //     println!("-->Client delete_ignore_version successfully");
    // };
    // match client_dao::select_all_include_deleted(&conn){
    //     Ok(clients) => {
    //         println!("-->Clients selected successfully:");
    //         for client in clients {
    //             println!("    {:?}", client);
    //         }
    //     }
    //     Err(err) => eprintln!("-->Error selecting clients: {}", err),
    // }

    //防止重启时，新进程还未完全启动，旧进程就已经退出了
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    println!("-->FINISH<--");
    Ok(())
}
