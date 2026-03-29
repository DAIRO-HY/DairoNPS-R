use crate::dao::channel_dao;
use crate::util::db_util;
use axum::{
    extract::Query,
    routing::delete,
    Router,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

/// 用来接收关闭通知的全局异步通知器
pub static SHUTDOWN_NOTIFY: Lazy<Arc<Notify>> = Lazy::new(|| Arc::new(Notify::const_new()));

pub static CHANNEL_CLOSE_NOTIFY: Lazy<Mutex<HashMap<i64, Arc<Notify>>>> = Lazy::new(|| Mutex::new(HashMap::new()));


/// 从查询参数中提取id
#[derive(Deserialize, Serialize, Debug)]
pub struct IdQuery {
    pub id: i64,
}

pub fn ready() {
    tokio::spawn(init_router());
}

async fn init_router() {
    let app = Router::new().route("/channel/delete", delete(delete_by));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:1880").await.unwrap();
    let _ = axum::serve(listener, app).await;
}

pub async fn delete_by(Query(query): Query<IdQuery>) {
    let mut conn = db_util::new_connection();
    let tx = conn.transaction().unwrap();
    let notify = CHANNEL_CLOSE_NOTIFY.lock().await.get(&query.id);
    //未完成
}
