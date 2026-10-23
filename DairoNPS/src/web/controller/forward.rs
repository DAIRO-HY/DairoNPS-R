use crate::dao::forward_dao::Forward;
use crate::dao::{forward_dao, traffic_stats_dao};
use crate::extension::number::NumberExtension;
use crate::web::biz_error::BizError;
use crate::web::form;
use crate::{biz_error, biz_errorf, forward};
use hyperoute_macro::{DELETE, GET, POST, PUT};
use std::collections::HashMap;
use validator::Validate;

/// 端口转发列表
#[GET("/api/forward/list")]
pub async fn list() -> Vec<model::ForwardList> {
    let mut ctx = lib_db::get_context();
    let forward_id_name_map = forward_dao::select_all(&mut ctx)
        .await
        .unwrap_or_default()
        .into_iter()
        .rev()
        .map(|it| (it.id, it.name))
        .collect::<HashMap<_, _>>();
    let list = forward_dao::select_all(&mut ctx)
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
    list
}

/// 端口转发详情获取API
#[GET("/api/forward/detail")]
pub async fn detail(id: i64) -> Result<model::ForwardDetail, BizError> {
    let mut ctx = lib_db::get_context();
    let detail = if id > 0 {
        let Ok(channel) = forward_dao::select_one(&mut ctx, id).await else {
            return Err(biz_error!("未找到隧道信息"));
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
    Ok(detail)
}

// Edit 提交表单API
#[POST("/api/forward/edit")]
pub async fn edit(form: form::ForwardEdit) -> Result<(), BizError> {
    if let Err(e) = form.validate() {
        //验证表单数据是否合法
        return Err(BizError::fields(e));
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
            return Err(biz_error!("未找到端口转发信息"));
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
            return Err(BizError::field(
                "serverPort",
                "该服务器端口已被其他隧道占用，请换一个端口。",
            ));
        }
        return Err(biz_error!(e.to_string()));
    }
    if forward.is_enabled {
        //当前隧道有效并且当前客户端在线，则开启隧道监听
        forward::tcp_accept::accept_forward(forward).await.unwrap();
    }
    Ok(())
}

/// 通过id删除一个端口转发
#[DELETE("/api/forward/delete")]
pub async fn delete(id: i64) -> Result<(), BizError> {
    //关闭端口转发监听(先关监听,统计任务就不会再产生这个端口转发的新数据)
    forward::tcp_accept::shutdown(id).await;

    //移除还没写入数据库的流量统计缓存,否则删除后还会写入孤儿数据
    forward::forward_timer::INSERT_CACHE_LIST
        .lock()
        .await
        .retain(|it| it.forward_id != id);
    let mut ctx = lib_db::get_context();
    ctx.begin().await.unwrap();
    let rs = async {
        forward_dao::delete(&mut ctx, id).await?;
        traffic_stats_dao::delete_by_forward_id(&mut ctx, id).await
    }
    .await;
    if let Err(e) = rs {
        let _ = ctx.rollback().await;
        return Err(biz_errorf!("删除失败:{}", e));
    }

    //提交事务
    let _ = ctx.commit().await;
    Ok(())
}

/// 修改可用状态
#[PUT("/api/forward/toggle_enable")]
pub async fn toggle_enable(id: i64) {
    let mut ctx = lib_db::get_context();
    let mut forward = forward_dao::select_one(&mut ctx, id).await.unwrap();
    forward_dao::toggle_enable(&mut ctx, id, !forward.is_enabled)
        .await
        .unwrap();
    if forward.is_enabled {
        //关闭端口转发监听
        forward::tcp_accept::shutdown(id).await;
    } else {
        forward.is_enabled = true;
        forward::tcp_accept::accept_forward(forward).await.unwrap();
    };
}

pub mod model {
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

}
