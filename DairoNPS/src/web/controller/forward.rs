use lib_axum_extract::response::AppError;
use crate::dao::forward_dao::Forward;
use crate::dao::{forward_dao, traffic_stats_dao};
use lib_axum_extract::response::{AppResponse, ResponseExt};
use crate::extension::number::NumberExtension;
use crate::forward::tcp_accept;
use lib_axum_extract::AppQuery;
use crate::web::router::IdQuery;
use axum::{
    Json,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use validator::Validate;
use crate::biz_error;

///  转发列表
pub async fn list() -> AppResponse {
    let list = forward_dao::select_all(&mut lib_db::get_context())
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
    Response::json(list)
}

/// 隧道详情获取API
pub async fn detail(AppQuery(query): AppQuery<IdQuery>) -> AppResponse {
    let conn = lib_db::get();
    let detail = if query.id > 0 {
        let Ok(channel) = forward_dao::select_one(&mut lib_db::get_context(), query.id).await else {
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
    Response::json(detail)
}

// Edit 提交表单API
pub async fn edit(form: model::ForwardEdit) -> AppResponse {
    if let Err(e) = form.validate() {
        //验证表单数据是否合法
        return Response::field_errors(e);
    }
    let mut ctx = lib_db::get_context();
    let mut forward = if form.id == 0 {
        Forward {
            is_enabled: true,
            ..Default::default()
        }
    } else {
        if let Ok(it) = forward_dao::select_one(&mut ctx, form.id).await {
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
        match forward_dao::insert(&mut ctx, &forward).await {
            Err(e) => err = Some(e),
            Ok(id) => forward.id = id,
        }
    } else {
        err = forward_dao::update(&mut ctx, &forward).await.err();
    }
    if let Some(e) = err {
        let err_msg = e.to_string();
        if err_msg == "UNIQUE constraint failed: channel.server_port" {
            return Response::field_error(
                "serverPort",
                "该服务器端口已被其他隧道占用，请换一个端口。",
            );
        }
        return biz_error!(err_msg);
    }
    if forward.is_enabled {
        //当前隧道有效并且当前客户端在线，则开启隧道监听
        tcp_accept::accept_forward(forward).await.unwrap();
    }
    Response::empty()
}

/// 通过id删除一个隧道
pub async fn delete(AppQuery(query): AppQuery<IdQuery>) -> AppResponse {
    let mut ctx = lib_db::get_context();
    ctx.begin().await?;
    forward_dao::delete(&mut ctx, query.id).await?;
    traffic_stats_dao::delete_by_forward_id(&mut ctx, query.id)
        .await?;

    //提交事务
    ctx.commit().await?;
    drop(ctx);

    // //关闭代理监听
    // udp_proxy.ShutdownByChannel(channel.Id)
    // tcp_proxy_manager::shutdown_by_channel(query.id).await;
    Response::empty()
}

/// 修改可用状态
pub async fn toggle_enable(AppQuery(query): AppQuery<IdQuery>) {
    let mut ctx = lib_db::get_context();
    let mut forward = forward_dao::select_one(&mut ctx, query.id).await.unwrap();
    forward_dao::toggle_enable(&mut ctx, query.id, !forward.is_enabled)
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
    use axum_request_macros::RequestForm;

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

    #[derive(Debug, Default, Serialize, Deserialize, RequestForm, Validate)]
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
