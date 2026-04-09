use crate::dao::client_dao;
use crate::dao::client_dao::Client;
use crate::extension::ResponseEmptyExt;
use crate::extension::number::ToDataSize;
use crate::extension::number::ToDateFormat;
use crate::nps::nps_client::tcp_client::tcp_client_session_manager;
use crate::web::extract::{AppForm, AppQuery};
use crate::web::router::IdQuery;
use crate::{biz_error, nps};
use axum::{
    Json,
    extract::Query,
    response::{IntoResponse, Response},
};
use rand::distr::{Alphanumeric, SampleString};
use std::collections::HashSet;
use validator::Validate;

/// 客户端列表
pub async fn list() -> Response {
    let online_client_set: HashSet<i64> = nps::CLIENT_NPS_MAP
        .lock()
        .await
        .keys()
        .map(|it| it.clone())
        .collect(); //收集在线状态的客户端id
    let list = client_dao::select_all(&db::get())
        .await
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
            in_len: it.in_len.data_size(),
            out_len: it.out_len.data_size(),
            is_online: online_client_set.contains(&it.id),
        })
        .collect::<Vec<_>>();
    Json(list).into_response()
}

/// 获取客户端详情API
pub async fn detail(Query(id): Query<IdQuery>) -> Response {
    let detail = if id.id > 0 {
        let Ok(client) = client_dao::select_one(&db::get(), id.id).await else {
            return biz_error!("未找到客户端信息");
        };
        model::ClientDetail {
            id: client.id,
            name: client.name,
            version: client.version,
            client_version: client.client_version,
            key: client.key,
            ip: client.ip,
            in_len: client.in_len.data_size(),
            out_len: client.out_len.data_size(),
            online_state: if nps::CLIENT_NPS_MAP.lock().await.contains_key(&client.id) {
                "在线".to_string()
            } else {
                "离线".to_string()
            },
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
    Json(detail).into_response()
}

// 提交表单API
pub async fn edit(AppForm(form): AppForm<model::ClientEdit>) -> Response {
    if let Err(e) = form.validate() {
        //验证表单数据是否合法
        return Response::field_errors(e);
    }

    let conn = db::get();
    let mut client = if form.id == 0 {
        Client {
            enable_state: 1,
            ..Default::default()
        }
    } else {
        if let Ok(it) = client_dao::select_one(&conn, form.id).await {
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

    let mut err = None;
    if form.id == 0 {
        if let Err(e) = client_dao::insert(&conn, client).await {
            err = Some(e);
        }
    } else {
        err = client_dao::update(&conn, client).await.err();
    }
    if let Some(e) = err {
        let err_msg = e.to_string();
        if err_msg == "UNIQUE constraint failed: client.key" {
            return Response::field_error("key", "该秘钥已被其他客户端使用，请换一个秘钥。");
        }
        return biz_error!(e.to_string());
    }

    //通知关闭该客户端会话
    tcp_client_session_manager::shutdown(form.id).await.unwrap();
    // application::IS_NEED_RESTART.store(true, std::sync::atomic::Ordering::Release);//标记需要重启
    Response::empty()
}

/// 通过id删除一个客户端
pub async fn delete(AppQuery(query): AppQuery<IdQuery>) {
    client_dao::set_delete_ignone_version(&db::get(), query.id)
        .await
        .unwrap();
    tcp_client_session_manager::shutdown(query.id)
        .await
        .unwrap();
    // application::IS_NEED_RESTART.store(true, std::sync::atomic::Ordering::Release);//标记需要重启
}

/// 修改可用状态
pub async fn toggle_enable(AppQuery(query): AppQuery<IdQuery>) {
    let conn = db::get();
    let client = client_dao::select_one(&conn, query.id).await.unwrap();
    let to = if client.enable_state == 0 {
        1
    } else {
        //关闭客户端
        tcp_client_session_manager::shutdown(query.id)
            .await
            .unwrap();
        0
    };
    client_dao::toggle_enable(&conn, query.id, to)
        .await
        .unwrap();
    // application::IS_NEED_RESTART.store(true, Ordering::Release);//标记需要重启
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
        pub enable_state: i64,
        pub in_len: String,
        pub out_len: String,
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
        pub in_len: String,

        // 出网流量
        pub out_len: String,

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
