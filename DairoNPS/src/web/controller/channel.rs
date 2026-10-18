use crate::dao::channel_dao::Channel;
use crate::dao::client_dao;
use crate::dao::{channel_dao, traffic_stats_dao};
use crate::extension::number::NumberExtension;
use crate::nps::nps_proxy::tcp_proxy;
use crate::web::biz_error::BizError;
use crate::web::form;
use crate::{biz_error, biz_errorf, nps};
use hyperoute_macro::{DELETE, GET, POST, PUT};
use std::collections::HashMap;
use validator::Validate;

///  隧道列表
#[GET("/api/channel/list")]
pub async fn list() -> Vec<model::ChannelList> {
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
    list
}

/// 隧道详情获取API
#[GET("/api/channel/detail")]
pub async fn detail(id: i64, client_id: i64) -> Result<model::ChannelDetail, BizError> {
    let mut ctx = lib_db::get_context();
    let detail = if id > 0 {
        let Ok(channel) = channel_dao::select_one(&mut ctx, id).await else {
            return Err(biz_error!("未找到隧道信息"));
        };
        model::ChannelDetail {
            id: channel.id,
            client_id: channel.client_id,
            client_name: "待实现".to_string(), //client_dao::select_one(&mut ctx, client.client_id).unwrap_or(Client::default()).name,
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
            client_id,
            mode: 1,
            ..model::ChannelDetail::default()
        }
    };
    Ok(detail)
}

// Edit 提交表单API
#[POST("/api/channel/edit")]
pub async fn edit(form: form::ChannelEdit) -> Result<(), BizError> {
    if let Err(e) = form.validate() {
        //验证表单数据是否合法
        return Err(BizError::fields(e));
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
            return Err(biz_error!("未找到隧道信息"));
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
        return Err(biz_error!("加密传输模式必须开启流量统计"));
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
            return Err(BizError::field(
                "serverPort",
                "该服务器端口已被其他隧道占用，请换一个端口。",
            ));
        }
        return Err(biz_error!(e.to_string()));
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
    Ok(())
}

/// 通过id删除一个隧道
#[DELETE("/api/channel/delete")]
pub async fn delete(id: i64) -> Result<(), BizError> {
    let mut ctx = lib_db::get_context();
    ctx.begin().await.unwrap();
    let rs = async {
        channel_dao::delete(&mut ctx, id).await?;
        traffic_stats_dao::delete_by_channel_id(&mut ctx, id).await
    }
    .await;
    if let Err(e) = rs {
        let _ = ctx.rollback().await;
        return Err(biz_errorf!("删除失败:{}", e));
    }

    //提交事务
    let _ = ctx.commit().await;
    drop(ctx);

    // //关闭代理监听
    // udp_proxy.ShutdownByChannel(channel.Id)
    tcp_proxy::shutdown_by_channel(id).await;
    Ok(())
}

/// 修改可用状态
#[PUT("/api/channel/toggle_enable")]
pub async fn toggle_enable(id: i64) {
    let mut ctx = lib_db::get_context();
    let mut channel = channel_dao::select_one(&mut ctx, id).await.unwrap();
    channel_dao::toggle_enable(&mut ctx, id, !channel.is_enabled)
        .await
        .unwrap();
    if channel.is_enabled {
        //关闭代理监听
        tcp_proxy::shutdown_by_channel(id).await;
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

pub mod model {
    use serde::{Deserialize, Serialize};
    use validator::Validate;

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

}
