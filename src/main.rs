mod constant;
mod dao;
mod entity;
mod nps;
mod util;
mod model;

use tokio::{io, try_join};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{Mutex, Notify},
};

use entity::channel::Channel;
use std::collections::HashMap;
use std::pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use dashmap::DashMap;

#[tokio::main]
async fn main() -> io::Result<()> {
    util::security_util::init();
    nps::init();

    tokio::spawn(async {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let hm = nps::BRIDGE_INFO.get().unwrap().lock().await;
            let Some(bridge_map) = hm.get(&0) else {
                println!("-->准备中");
                continue;
            };
            println!(
                "-->当前桥接数量: {}",bridge_map.len()
            );

            for it in bridge_map.iter() {
                let data_total = it.value().data_total.clone();
                println!("桥接ID: {} 入流量统计: {} 出流量统计: {}", it.key(), data_total.load_in(),data_total.load_out());
            }
            // nps::nps_pool::tcp_pool_manager::shutdown_by_client(0).await;
        }
    });


    println!("-->START<--");
    nps::nps_client::tcp_client::tcp_client_accept::accept().await?;
    println!("-->FINISH<--");
    Ok(())
}
