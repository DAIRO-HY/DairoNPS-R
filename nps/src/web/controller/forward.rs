use crate::dao::forward_dao::Forward;
use crate::dao::{forward_dao, traffic_stats_dao};
use crate::extension::ResponseEmptyExt;
use crate::extension::number::ToDataSize;
use crate::extension::number::ToDateFormat;
use crate::forward::tcp_accept;
use crate::nps::nps_proxy::tcp_proxy;
use crate::web::extract::{AppForm, AppQuery};
use crate::web::router::IdQuery;
use crate::{biz_error, biz_errorf, nps};
use axum::{
    Json,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use validator::Validate;

///  转发列表
pub async fn list() -> Response {
    let conn = db::get();
    let forward_id_name_map = forward_dao::select_all(&conn)
        .await
        .unwrap_or_default()
        .into_iter()
        .rev()
        .map(|it| (it.id, it.name))
        .collect::<HashMap<_, _>>();
    let list = forward_dao::select_all(&conn)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|it| model::ForwardList {
            id: it.id,
            name: it.name,
            server_port: it.server_port,
            target_port: it.target_port,
            is_enabled: it.is_enabled,
            in_len: it.in_len.data_size(),
            out_len: it.out_len.data_size(),
            error: it.error,
        })
        .collect::<Vec<_>>();
    Json(list).into_response()
}

/// 隧道详情获取API
pub async fn detail(AppQuery(query): AppQuery<IdQuery>) -> Response {
    let conn = db::get();
    let detail = if query.id > 0 {
        let Ok(channel) = forward_dao::select_one(&conn, query.id).await else {
            return biz_error!("未找到隧道信息");
        };
        model::ForwardDetail {
            id: channel.id,
            name: channel.name,
            server_port: channel.server_port,
            target_port: channel.target_port,
            remark: channel.remark,
            created_at: channel.created_at.date_format(),
            is_enabled: if channel.is_enabled {
                "开启"
            } else {
                "关闭"
            }
            .to_string(),
            is_stats_traffic: channel.is_stats_traffic,
            in_len: channel.in_len.data_size(),
            out_len: channel.out_len.data_size(),
        }
    } else {
        model::ForwardDetail::default()
    };
    Json(detail).into_response()
}

// Edit 提交表单API
pub async fn edit(AppForm(form): AppForm<model::ForwardEdit>) -> Response {
    if let Err(e) = form.validate() {
        //验证表单数据是否合法
        return Response::field_errors(e);
    }
    let conn = db::get();
    let mut forward = if form.id == 0 {
        Forward {
            is_enabled: true,
            ..Default::default()
        }
    } else {
        if let Ok(it) = forward_dao::select_one(&conn, form.id).await {
            it
        } else {
            return biz_error!("未找到端口转发信息");
        }
    };
    forward.id = form.id;
    forward.name = form.name;
    forward.server_port = form.server_port;
    forward.target_port = form.target_port;
    forward.is_stats_traffic = form.is_stats_traffic;
    forward.remark = form.remark;

    let mut err = None;
    if form.id == 0 {
        match forward_dao::insert(&conn, &forward).await {
            Err(e) => err = Some(e),
            Ok(id) => forward.id = id,
        }
    } else {
        err = forward_dao::update(&conn, &forward).await.err();
    }
    if let Some(e) = err {
        let err_msg = e.to_string();
        if err_msg == "UNIQUE constraint failed: channel.server_port" {
            return Response::field_error(
                "serverPort",
                "该服务器端口已被其他隧道占用，请换一个端口。",
            );
        }
        return biz_error!(e.to_string());
    }
    if forward.is_enabled {
        //当前隧道有效并且当前客户端在线，则开启隧道监听
        tcp_accept::accept_forward(forward).await.unwrap();
    }
    Response::empty()
}

/// 通过id删除一个隧道
pub async fn delete(AppQuery(query): AppQuery<IdQuery>) -> Response {
    let conn = db::get();
    let mut tx = conn.begin().await.unwrap();
    if let Err(e) = forward_dao::delete(&mut *tx, query.id).await {
        return biz_errorf!("删除失败:{}", e);
    }
    traffic_stats_dao::delete_by_forward_id(&mut *tx, query.id)
        .await
        .unwrap();

    //提交事务
    let _ = tx.commit().await;
    drop(conn);

    // //关闭代理监听
    // udp_proxy.ShutdownByChannel(channel.Id)
    // tcp_proxy_manager::shutdown_by_channel(query.id).await;
    Response::empty()
}

/// 修改可用状态
pub async fn toggle_enable(AppQuery(query): AppQuery<IdQuery>) {
    let conn = db::get();
    let mut forward = forward_dao::select_one(&conn, query.id).await.unwrap();
    forward_dao::toggle_enable(&conn, query.id, !forward.is_enabled)
        .await
        .unwrap();
    if forward.is_enabled {
        //关闭代理监听
        tcp_accept::shutdown(query.id).await;
    } else {
        forward.is_enabled = true;
        tcp_accept::accept_forward(forward).await.unwrap();
    };
}

mod model {
    use serde::{Deserialize, Serialize};
    use validator::Validate;

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ForwardList {
        pub id: i64,
        pub name: String,
        pub server_port: i64,
        pub target_port: String,
        pub is_enabled: bool,
        pub in_len: String,
        pub out_len: String,
        pub error: Option<String>,
    }

    #[derive(Debug, Serialize, Default)]
    #[serde(rename_all = "camelCase")]
    pub struct ForwardDetail {
        pub id: i64,
        pub name: String,
        pub server_port: i64,
        pub target_port: String,
        pub remark: Option<String>,
        pub created_at: String,
        pub is_enabled: String,

        ///是否统计流量
        pub is_stats_traffic: bool,
        pub in_len: String,
        pub out_len: String,
    }

    #[derive(Debug, Default, Serialize, Deserialize, Validate)]
    #[serde(rename_all = "camelCase")]
    pub struct ForwardEdit {
        pub id: i64,

        #[validate(length(min = 1, max = 32, message = "名称不能为空;长度不能超过32位"))]
        pub name: String,
        pub server_port: i64,
        pub target_port: String,

        ///是否统计流量
        pub is_stats_traffic: bool,
        pub remark: Option<String>,
    }
}
