use lib_axum_extract::response::AppError;
use crate::dao::channel_dao::Channel;
use crate::dao::client_dao;
use crate::dao::{channel_dao, traffic_stats_dao};
use lib_axum_extract::response::{AppResponse, ResponseExt};
use crate::extension::number::NumberExtension;
use crate::nps::nps_proxy::tcp_proxy;
use lib_axum_extract::AppQuery;
use crate::web::router::IdQuery;
use crate::{biz_error, nps};
use axum::{
    Json,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use validator::Validate;

///  隧道列表
pub async fn list() -> AppResponse {
    let mut ctx = lib_db::get_context();
    let client_id_name_map = client_dao::select_all(&mut ctx)
        .await
        .unwrap_or_default()
        .into_iter()
        .rev()
        .map(|it| (it.id, it.name))
        .collect::<HashMap<_, _>>();
    let list = channel_dao::select_all(&mut ctx)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(|it| model::ChannelList {
            id: it.id,
            client_id: it.client_id,
            client_name: client_id_name_map
                .get(&it.client_id)
                .cloned()
                .unwrap_or_else(|| "未知".to_string()),
            name: it.name,
            mode: it.mode,
            server_port: it.server_port,
            target_port: it.target_port,
            is_enabled: it.is_enabled,
            in_len: it.in_len.data_size(),
            out_len: it.out_len.data_size(),
            security_state: it.security_state,
            error: it.error,
        })
        .collect::<Vec<_>>();
    Response::json(list)
}

/// 隧道详情获取API
pub async fn detail(AppQuery(query): AppQuery<model::DetailQuery>) -> AppResponse {
    let detail = if query.id > 0 {
        let Ok(channel) = channel_dao::select_one(&mut lib_db::get_context(), query.id).await else {
            return biz_error!("未找到隧道信息");
        };
        model::ChannelDetail {
            id: channel.id,
            client_id: channel.client_id,
            client_name: "待实现".to_string(), //client_dao::select_one(&conn, client.client_id).unwrap_or(Client::default()).name,
            name: channel.name,
            mode: channel.mode,
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
            security_state: channel.security_state,
            version: channel.version,
        }
    } else {
        model::ChannelDetail {
            client_id: query.client_id,
            mode: 1,
            ..model::ChannelDetail::default()
        }
    };
    Response::json(detail)
}

// Edit 提交表单API
pub async fn edit(form: model::ChannelEdit) -> AppResponse {
    if let Err(e) = form.validate() {
        //验证表单数据是否合法
        return Response::field_errors(e);
    }
    let mut ctx = lib_db::get_context();
    let mut channel = if form.id == 0 {
        Channel {
            is_enabled: true,
            ..Default::default()
        }
    } else {
        if let Ok(it) = channel_dao::select_one(&mut ctx, form.id).await {
            it
        } else {
            return biz_error!("未找到隧道信息");
        }
    };

    channel.id = form.id;
    channel.client_id = form.client_id;
    channel.name = form.name;
    channel.mode = form.mode;
    channel.server_port = form.server_port;
    channel.target_port = form.target_port;
    channel.security_state = form.security_state;
    channel.is_stats_traffic = form.is_stats_traffic;
    channel.remark = form.remark;
    channel.version = form.version;

    if channel.security_state == 1 && !channel.is_stats_traffic{
        return biz_error!("加密传输模式必须开启流量统计");
    }

    let mut err = None;
    if form.id == 0 {
        match channel_dao::insert(&mut ctx, &channel).await {
            Err(e) => err = Some(e),
            Ok(id) => channel.id = id,
        }
    } else {
        err = channel_dao::update(&mut ctx, &channel).await.err();
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
    if channel.is_enabled
        && nps::CLIENT_LIVE_MAP
            .lock()
            .await
            .contains_key(&channel.client_id)
    {
        //当前隧道有效并且当前客户端在线，则开启隧道监听
        tcp_proxy::ready_channel(channel).await.unwrap(); //开启隧道监听
    }
    Response::empty()
}

/// 通过id删除一个隧道
pub async fn delete(AppQuery(query): AppQuery<IdQuery>) -> AppResponse {
    let mut ctx = lib_db::get_context();
    ctx.begin().await?;
    channel_dao::delete(&mut ctx, query.id).await?;
    traffic_stats_dao::delete_by_channel_id(&mut ctx, query.id)
        .await?;

    //提交事务
    ctx.commit().await;
    drop(ctx);

    // //关闭代理监听
    // udp_proxy.ShutdownByChannel(channel.Id)
    tcp_proxy::shutdown_by_channel(query.id).await;
    Response::empty()
}

/// 修改可用状态
pub async fn toggle_enable(AppQuery(query): AppQuery<IdQuery>) {
    let mut ctx = lib_db::get_context();
    let mut channel = channel_dao::select_one(&mut ctx, query.id).await.unwrap();
    channel_dao::toggle_enable(&mut ctx, query.id, !channel.is_enabled)
        .await
        .unwrap();
    if channel.is_enabled {
        //关闭代理监听
        tcp_proxy::shutdown_by_channel(query.id).await;
    } else {
        if nps::CLIENT_LIVE_MAP
            .lock()
            .await
            .contains_key(&channel.client_id)
        {
            channel.is_enabled = true;

            //如果当前客户端在线
            tcp_proxy::ready_channel(channel).await.unwrap(); //开启隧道监听
        }
    };
}

mod model {
    use serde::{Deserialize, Serialize};
    use validator::Validate;
    use axum_request_macros::RequestForm;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ChannelListQuery {
        pub client_id: i64,
    }

    #[derive(Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ChannelList {
        pub id: i64,
        pub client_id: i64,
        pub client_name: String,
        pub name: String,
        pub mode: i64,
        pub server_port: i64,
        pub target_port: String,
        pub is_enabled: bool,
        pub in_len: String,
        pub out_len: String,
        pub security_state: i64,
        pub error: Option<String>,
    }

    /// 隧道详细信息获取时的请求参数
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DetailQuery {
        pub id: i64,
        pub client_id: i64,
    }

    #[derive(Debug, Serialize, Default)]
    #[serde(rename_all = "camelCase")]
    pub struct ChannelDetail {
        pub id: i64,
        pub client_id: i64,
        pub client_name: String,
        pub name: String,
        pub mode: i64,
        pub server_port: i64,
        pub target_port: String,
        pub remark: Option<String>,
        pub created_at: String,
        pub is_enabled: String,
        pub in_len: String,
        pub out_len: String,
        pub security_state: i64,

        ///是否统计流量
        pub is_stats_traffic: bool,

        // 乐观排他用的版本号
        pub version: i64,
    }

    #[derive(Debug, Default, Serialize, Deserialize, RequestForm, Validate)]
    #[serde(rename_all = "camelCase")]
    pub struct ChannelEdit {
        pub id: i64,
        pub client_id: i64,

        #[validate(length(min = 1, max = 32, message = "名称不能为空;长度不能超过32位"))]
        pub name: String,
        pub mode: i64,
        pub server_port: i64,
        pub target_port: String,
        pub security_state: i64,

        ///是否统计流量
        pub is_stats_traffic: bool,
        pub remark: Option<String>,

        // 乐观排他用的版本号
        pub version: i64,
    }
}
