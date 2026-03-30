use crate::constant::nps_constant;
use crate::dao::client_dao;
use crate::dao::client_dao::Client;
use crate::extension::ResponseEmptyExt;
use crate::extension::number::ToDataSize;
use crate::extension::number::ToDateFormat;
use crate::util::db_util;
use crate::web::extract::{AppForm, AppQuery};
use crate::web::router::IdQuery;
use crate::{biz_error, biz_errorf};
use axum::{
    Json,
    extract::Query,
    response::{IntoResponse, Response},
};
use rand::distr::{Alphanumeric, SampleString};
use validator::Validate;
use std::sync::atomic::Ordering;
use crate::application;

/// 客户端列表
pub async fn list() -> Response {
    let conn = db_util::new_connection();
    let list = client_dao::select_all(&conn)
        .unwrap_or_default()
        .into_iter()
        .rev()
        .map(|it| model::ClitentList {
            id: it.id,
            name: it.name,
            key: it.key,
            ip: it.ip.unwrap_or("未连接".to_string()),
            client_version: it.client_version.unwrap_or("未连接".to_string()),
            enable_state: it.enable_state,
            in_data: it.in_data.data_size(),
            out_data: it.out_data.data_size(),
            // is_online: crate::nps_client::tcp_client::is_online(it.id),
            is_online: false, //待实现
        })
        .collect::<Vec<_>>();
    Json(list).into_response()
}

/// 获取客户端详情API
pub async fn detail(Query(id): Query<IdQuery>) -> Response {
    let conn = db_util::new_connection();
    let detail = if id.id > 0 {
        let Ok(client) = client_dao::select_one(&conn, id.id) else {
            return biz_error!("未找到客户端信息");
        };
        model::ClientDetail {
            id: client.id,
            name: client.name,
            version: client.version,
            client_version: client.client_version,
            key: client.key,
            ip: client.ip,
            in_data: client.in_data.data_size(),
            out_data: client.out_data.data_size(),
            // online_state: if crate::nps_client::tcp_client::IsOnline(client.id) { "在线" } else { "离线" }.to_string(),
            online_state: "待实现".to_string(),
            enable_state: if client.enable_state == 0 {
                "关闭"
            } else {
                "开启"
            }
            .to_string(),
            last_login_date: client.last_login_date.date_format(),
            created_at: client.created_at.date_format(),
            updated_at: client.updated_at.date_format(),
            remark: client.remark,
        }
    } else {
        model::ClientDetail {
            key: Alphanumeric.sample_string(&mut rand::rng(), 16),
            ..Default::default()
        }
    };
    return Json(detail).into_response();
}

// 提交表单API
pub async fn edit(AppForm(form): AppForm<model::ClientEdit>) -> Response {
    if let Err(e) = form.validate() {
        //验证表单数据是否合法
        return Response::field_errors(e);
    }

    let conn = db_util::new_connection();
    let mut client = if form.id == 0 {
        Client{
            enable_state:1,
            ..Default::default()
        }
    } else {
        if let Ok(it) = client_dao::select_one(&conn, form.id) {
            it
        } else {
            return biz_error!("未找到客户端信息");
        }
    };

    client.id = form.id;
    client.name = form.name; //名称
    client.version = form.version; //乐观排他用的版本号
    client.key = form.key; //连接认证秘钥
    client.remark = form.remark; //一些备注信息,错误信息等
    client.enable_state = 1; //启用状态

    let conn = db_util::new_connection();
    let mut err = None;
    if form.id == 0 {
        if let Err(e) = client_dao::insert(&conn, client) {
            err = Some(e);
        }
    } else {
        err = client_dao::update(&conn, client);
    }
    // tcp_client.Shutdown(form.Id)
    if let Some(e) = err {
        let err_msg = e.to_string();
        if err_msg == "UNIQUE constraint failed: client.key" {
            return Response::field_error("key", "该秘钥已被其他客户端使用，请换一个秘钥。");
        }
        return biz_error!(e.to_string());
    }
    crate::application::IS_NEED_RESTART.store(true, std::sync::atomic::Ordering::Release);//标记需要重启
    Response::empty()
}

/// 通过id删除一个客户端
pub async fn delete(AppQuery(query): AppQuery<IdQuery>) {
    let conn = db_util::new_connection();
    client_dao::delete_ignore_version(&conn, query.id);
    crate::application::IS_NEED_RESTART.store(true, std::sync::atomic::Ordering::Release);//标记需要重启
}

/// 修改可用状态
pub async fn toggle_enable(AppQuery(query): AppQuery<IdQuery>) {
    let conn = db_util::new_connection();
	let client = client_dao::select_one(&conn, query.id).unwrap();
	let to = if client.enable_state == 0 {
        1
	} else {
        0
	};
    client_dao::toggle_enable(&conn, query.id, to);
    application::IS_NEED_RESTART.store(true, Ordering::Release);//标记需要重启
}

mod model {
    use serde::{Deserialize, Serialize};
    use validator::Validate;

    #[derive(Debug, Default, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ClitentList {
        pub id: i64,
        pub name: String,
        pub key: String,
        pub ip: String,
        pub client_version: String,
        pub enable_state: i8,
        pub in_data: String,
        pub out_data: String,
        pub is_online: bool,
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
        pub in_data: String,

        // 出网流量
        pub out_data: String,

        // 在线状态
        pub online_state: String,

        // 启用状态
        pub enable_state: String,

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
