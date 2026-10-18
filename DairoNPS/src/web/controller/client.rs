use crate::dao::client_dao;
use crate::dao::client_dao::Client;
use crate::extension::number::NumberExtension;
use crate::web::biz_error::BizError;
use crate::web::form;
use crate::web::model::IdQuery;
use crate::{biz_error, nps};
use hyperoute_core::Query;
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
    let list = client_dao::select_all(&lib_db::get())
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
pub async fn detail(id: Query<IdQuery>) -> Result<model::ClientDetail, BizError> {
    let detail = if id.id > 0 {
        let Ok(client) = client_dao::select_one(&lib_db::get(), id.id).await else {
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

    let conn = lib_db::get();
    let mut client = if form.id == 0 {
        Client {
            is_enabled: true,
            ..Default::default()
        }
    } else {
        if let Ok(it) = client_dao::select_one(&conn, form.id).await {
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
    client.is_enabled = true; //启用状态

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
pub async fn delete(query: Query<IdQuery>) {
    client_dao::delete(&lib_db::get(), query.id)
        .await
        .unwrap();
    nps_session::shutdown(query.id)
        .await
        .unwrap();
}

/// 修改可用状态
#[PUT("/api/client/toggle_enable")]
pub async fn toggle_enable(query: Query<IdQuery>) {
    let conn = lib_db::get();
    let client = client_dao::select_one(&conn, query.id).await.unwrap();
    let to = if client.is_enabled {
        //关闭客户端
        nps_session::shutdown(query.id)
            .await
            .unwrap();
        false
    } else {
        true
    };
    client_dao::toggle_enable(&conn, query.id, to)
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
