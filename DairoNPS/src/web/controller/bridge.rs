use crate::dao::{channel_dao, client_dao, forward_dao};
use crate::extension::number::NumberExtension;
use crate::nps::TCPBridging;
use crate::web::controller::bridge::model::BridgeList;
use crate::web::router::SingleQuery;
use crate::{forward, nps};
use axum::{
    response::{IntoResponse, Response},
};
use lib_axum_extract::AppQuery;
use lib_axum_extract::response::{AppResponse, ResponseExt};
use lib_np_common::time_util;
use rand::distr::SampleString;
use sqlx_context::DbContext;
use std::cmp::Ordering::{Equal, Greater, Less};
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use validator::Validate;

/// 客户端列表
pub async fn list(AppQuery(param): AppQuery<SingleQuery<String>>) -> AppResponse {
    let mut ctx = lib_db::get_context();
    let now = time_util::current_millis();
    let mut list = Vec::new();
    if param.value == "nps" {
        list.extend(nps_list(&mut ctx, now).await);
    } else if param.value == "forward" {
        list.extend(forward_list(&mut ctx, now).await);
    } else {
        list.extend(nps_list(&mut ctx, now).await);
        list.extend(forward_list(&mut ctx, now).await);
    }
    list.sort_by(|p1, p2| {
        if p1.alive_time > p2.alive_time {
            Less
        } else if (p1.alive_time < p2.alive_time) {
            Greater
        } else {
            Equal
        }
    });
    Response::json(list)
}

async fn nps_list(ctx: &mut DbContext, now: u64) -> Vec<BridgeList> {
    let client_id_name: HashMap<i64, String> = client_dao::select_all(&mut *ctx)
        .await
        .unwrap_or_default()
        .iter()
        .map(|it| (it.id, it.name.clone()))
        .collect();

    //隧道id对应的名称
    let channel_id_name: HashMap<i64, String> = channel_dao::select_all(&mut *ctx)
        .await
        .unwrap_or_default()
        .iter()
        .map(|it| {
            let client_name = client_id_name
                .get(&it.client_id)
                .map(String::as_str)
                .unwrap_or_default();
            (it.id, format!("{}-{}", client_name, it.name))
        })
        .collect();

    // 内网穿透的桥接列表
    let mut nps_bridges: Vec<(u64, TCPBridging)> = nps::CHANNEL_BRIDGING_MAP
        .iter()
        .map(|it| (it.key().clone(), it.value().clone()))
        .collect();

    nps_bridges.sort_by_key(|(_, it)| it.create_time);
    // nps_bridges.reverse(); //倒序（反转）
    nps_bridges.truncate(100); //只取前100条数据
    nps_bridges
        .iter()
        .map(|(tag, it)| BridgeList {
            tag: *tag,
            name: channel_id_name
                .get(&it.channel_id)
                .map(String::to_string)
                .unwrap_or_default(),
            ip: it.ip.clone(),
            in_len: it.data_len.load_in().data_size(),
            out_len: it.data_len.load_out().data_size(),
            alive_time: (now - it.create_time).time_format(),
            idle_time: (now - it.last_rw_time.load(Ordering::Relaxed)).time_format(),
        })
        .collect::<Vec<_>>()
}

async fn forward_list(ctx: &mut DbContext, now: u64) -> Vec<BridgeList> {
    //端口转发id对应的名称
    let forward_id_name: HashMap<i64, String> = forward_dao::select_all(&mut *ctx)
        .await
        .unwrap_or_default()
        .iter()
        .map(|it| (it.id, it.name.clone()))
        .collect();

    // 端口转发的桥接列表
    let mut forward_bridges: Vec<(u64, forward::TCPBridging)> = forward::FORWARD_BRIDGING_MAP
        .iter()
        .map(|it| (it.key().clone(), it.value().clone()))
        .collect();

    forward_bridges.sort_by_key(|(_, it)| it.create_time);
    // forward_bridges.forward_bridges(); //倒序（反转）
    forward_bridges.truncate(100); //只取前100条数据
    forward_bridges
        .iter()
        .map(|(tag, it)| BridgeList {
            tag: *tag,
            name: forward_id_name
                .get(&it.forward_id)
                .map(String::to_string)
                .unwrap_or_default(),
            ip: it.ip.clone(),
            in_len: it.data_len.load_in().data_size(),
            out_len: it.data_len.load_out().data_size(),
            alive_time: (now - it.create_time).time_format(),
            idle_time: (now - it.last_rw_time.load(Ordering::Relaxed)).time_format(),
        })
        .collect::<Vec<_>>()
}

/// 强制中断
pub async fn broken(AppQuery(param): AppQuery<SingleQuery<u64>>) -> AppResponse {
    if let Some(bridging) = nps::CHANNEL_BRIDGING_MAP.get(&param.value) {
        bridging.closer.notify_waiters();
    } else if let Some(bridging) = forward::FORWARD_BRIDGING_MAP.get(&param.value) {
        bridging.closer.notify_waiters();
    } else {
    }
    Response::empty()
}

mod model {
    use serde::{Deserialize, Serialize};
    use validator::Validate;

    #[derive(Debug, Default, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct BridgeList {
        pub tag: u64,
        pub name: String,
        pub ip: String,
        pub in_len: String,
        pub out_len: String,
        pub alive_time: String, //存活时间
        pub idle_time: String,  //空闲时间
    }

    // 客户端编辑信息
    #[derive(Debug, Default, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ClientDetail {
        // id
        pub id: i64,

        // 名称
        pub name: String,

        // 乐观排他用的版本号
        pub version: i64,

        // 客户端版本
        pub client_version: Option<String>,

        // 连接认证秘钥
        pub key: String,

        // ip地址
        pub ip: Option<String>,

        // 入网流量
        pub in_len: String,

        // 出网流量
        pub out_len: String,

        // 在线状态
        pub online_state: String,

        // 启用状态
        pub is_enabled: String,

        // 最后一次连接时间
        pub last_login_date: String,

        // 创建时间
        pub created_at: String,

        // 最后一次更新时间
        pub updated_at: String,

        // 一些备注信息,错误信息等
        pub remark: Option<String>,
    }

    // 客户端编辑信息
    #[derive(Deserialize, Debug, Validate)]
    #[serde(rename_all = "camelCase")]
    pub struct ClientEdit {
        // id
        pub id: i64,

        // 名称
        #[validate(length(min = 1, max = 32, message = "名称不能为空;长度不能超过32位"))]
        pub name: String,

        // 乐观排他用的版本号
        pub version: i64,

        // 连接认证秘钥
        #[validate(length(min = 1, max = 32, message = "认证秘钥不能为空;长度不能超过32位"))]
        pub key: String,

        // 一些备注信息,错误信息等
        pub remark: Option<String>,
    }
}
