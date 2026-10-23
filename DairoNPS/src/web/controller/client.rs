use crate::dao::client_dao::Client;
use crate::dao::{channel_dao, client_dao, traffic_stats_dao};
use crate::extension::number::NumberExtension;
use crate::nps::nps_proxy::tcp_proxy;
use crate::web::biz_error::BizError;
use crate::web::form;
use crate::{biz_error, biz_errorf, nps};
use hyperoute_macro::{DELETE, GET, POST, PUT};
use rand::distr::{Alphanumeric, SampleString};
use std::collections::HashSet;
use validator::Validate;
use crate::nps::nps_client::nps_session;

/// 客户端列表
#[GET("/api/client/list")]
pub async fn list() -> Vec<model::ClitentList> {
    let online_client_set: HashSet<i64> = nps::CLIENT_LIVE_MAP
        .lock()
        .await
        .keys()
        .map(|it| it.clone())
        .collect(); //收集在线状态的客户端id
    let list = client_dao::select_all(&mut lib_db::get_context())
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
            is_enabled: it.is_enabled,
            in_len: it.in_len.data_size(),
            out_len: it.out_len.data_size(),
            is_online: online_client_set.contains(&it.id),
        })
        .collect::<Vec<_>>();
    list
}

/// 获取客户端详情API
#[GET("/api/client/detail")]
pub async fn detail(id: i64) -> Result<model::ClientDetail, BizError> {
    let detail = if id > 0 {
        let Ok(client) = client_dao::select_one(&mut lib_db::get_context(), id).await else {
            return Err(biz_error!("未找到客户端信息"));
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
            online_state: if nps::CLIENT_LIVE_MAP.lock().await.contains_key(&client.id) {
                "在线".to_string()
            } else {
                "离线".to_string()
            },
            is_enabled: if client.is_enabled {
                "开启"
            } else {
                "关闭"
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
    Ok(detail)
}

// 提交表单API
#[POST("/api/client/edit")]
pub async fn edit(form: form::ClientEdit) -> Result<(), BizError> {
    if let Err(e) = form.validate() {
        //验证表单数据是否合法
        return Err(BizError::fields(e));
    }

    let mut ctx = lib_db::get_context();
    let mut client = if form.id == 0 {
        Client {
            is_enabled: true,
            ..Default::default()
        }
    } else {
        if let Ok(it) = client_dao::select_one(&mut ctx, form.id).await {
            it
        } else {
            return Err(biz_error!("未找到客户端信息"));
        }
    };

    client.id = form.id;
    client.name = form.name; //名称
    client.version = form.version; //乐观排他用的版本号
    client.key = form.key; //连接认证秘钥
    client.remark = form.remark; //一些备注信息,错误信息等

    let mut err = None;
    if form.id == 0 {
        if let Err(e) = client_dao::insert(&mut ctx, client).await {
            err = Some(e);
        }
    } else {
        err = client_dao::update(&mut ctx, client).await.err();
    }
    if let Some(e) = err {
        let err_msg = e.to_string();
        if err_msg == "UNIQUE constraint failed: client.key" {
            return Err(BizError::field("key", "该秘钥已被其他客户端使用，请换一个秘钥。"));
        }
        return Err(biz_error!(e.to_string()));
    }

    //通知关闭该客户端会话
    nps_session::shutdown(form.id).await.unwrap();
    Ok(())
}

/// 通过id删除一个客户端
#[DELETE("/api/client/delete")]
pub async fn delete(id: i64) -> Result<(), BizError> {
    //关闭客户端会话(先关会话,统计任务就不会再产生这个客户端的新数据)
    let _ = nps_session::shutdown(id).await;

    //关闭会话只是通知了隧道监听关闭,这里要等到该客户端下所有隧道监听真正停止
    let channel_ids: Vec<i64> = nps::CHANNEL_LIVE_MAP
        .lock()
        .await
        .iter()
        .filter(|(_, channel_live)| channel_live.client_id == id)
        .map(|(channel_id, _)| *channel_id)
        .collect();
    for channel_id in channel_ids {
        tcp_proxy::shutdown_by_channel(channel_id).await;
    }

    //移除还没写入数据库的流量统计缓存,否则删除后还会写入孤儿数据
    nps::nps_timer::INSERT_CACHE_LIST
        .lock()
        .await
        .retain(|it| it.client_id != id);
    let mut ctx = lib_db::get_context();
    ctx.begin().await.unwrap();
    let rs = async {
        //删除客户端的流量统计(包含该客户端下所有隧道的流量统计)
        traffic_stats_dao::delete_by_client_id(&mut ctx, id).await?;

        //删除客户端下的所有隧道
        channel_dao::delete_by_client_id(&mut ctx, id).await?;
        client_dao::delete(&mut ctx, id).await
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
#[PUT("/api/client/toggle_enable")]
pub async fn toggle_enable(id: i64) {
    let mut ctx = lib_db::get_context();
    let client = client_dao::select_one(&mut ctx, id).await.unwrap();
    let to = if client.is_enabled {
        //关闭客户端
        nps_session::shutdown(id)
            .await
            .unwrap();
        false
    } else {
        true
    };
    client_dao::toggle_enable(&mut ctx, id, to)
        .await
        .unwrap();
}

pub mod model {
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
        pub is_enabled: bool,
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

}
