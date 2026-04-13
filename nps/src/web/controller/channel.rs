use crate::dao::channel_dao::Channel;
use crate::dao::client_dao;
use crate::dao::{channel_dao, channel_data_dao};
use crate::extension::ResponseEmptyExt;
use crate::extension::number::ToDataSize;
use crate::extension::number::ToDateFormat;
use crate::nps::nps_proxy::tcp_proxy_manager;
use crate::web::extract::{AppForm, AppQuery};
use crate::web::router::IdQuery;
use crate::{biz_error, biz_errorf, nps};
use axum::{
    Json,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use validator::Validate;

///  隧道列表
pub async fn list() -> Response {
    let conn = db::get();
    let client_id_name_map = client_dao::select_all(&conn)
        .await
        .unwrap_or_default()
        .into_iter()
        .rev()
        .map(|it| (it.id, it.name))
        .collect::<HashMap<_, _>>();
    let list = channel_dao::select_all(&conn)
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
    Json(list).into_response()
}

/// 隧道详情获取API
pub async fn detail(AppQuery(query): AppQuery<model::DetailQuery>) -> Response {
    let conn = db::get();
    let detail = if query.id > 0 {
        let Ok(channel) = channel_dao::select_one(&conn, query.id).await else {
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
    Json(detail).into_response()
}

// Edit 提交表单API
// post:/channel_list/channel_edit/edit
pub async fn edit(AppForm(form): AppForm<model::ChannelEdit>) -> Response {
    if let Err(e) = form.validate() {
        //验证表单数据是否合法
        return Response::field_errors(e);
    }
    let conn = db::get();
    let mut channel = if form.id == 0 {
        Channel {
            is_enabled: true,
            ..Default::default()
        }
    } else {
        if let Ok(it) = channel_dao::select_one(&conn, form.id).await {
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
    channel.remark = form.remark;
    channel.version = form.version;

    let mut err = None;
    if form.id == 0 {
        match channel_dao::insert(&conn, &channel).await {
            Err(e) => err = Some(e),
            Ok(id) => channel.id = id,
        }
    } else {
        err = channel_dao::update(&conn, &channel).await.err();
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
    if channel.is_enabled
        && nps::CLIENT_NPS_MAP
            .lock()
            .await
            .contains_key(&channel.client_id)
    {
        //当前隧道有效并且当前客户端在线，则开启隧道监听
        tcp_proxy_manager::accept_channel(channel).await; //开启隧道监听
    }
    // crate::application::restart_mark();//标记需要重启
    Response::empty()
}

/// 通过id删除一个隧道
pub async fn delete(AppQuery(query): AppQuery<IdQuery>) -> Response {
    let conn = db::get();
    let mut tx = conn.begin().await.unwrap();
    println!("-->id:{}", query.id);
    if let Err(e) = channel_dao::delete(&mut *tx, query.id).await {
        return biz_errorf!("删除失败:{}", e);
    }
    channel_data_dao::delete_by_channel_id(&mut *tx, query.id)
        .await
        .unwrap();

    //提交事务
    let _ = tx.commit().await;
    drop(conn);

    // //关闭代理监听
    // udp_proxy.ShutdownByChannel(channel.Id)
    tcp_proxy_manager::shutdown_by_channel(query.id).await;
    Response::empty()
}

/// 修改可用状态
pub async fn toggle_enable(AppQuery(query): AppQuery<IdQuery>) {
    let conn = db::get();
    let channel = channel_dao::select_one(&conn, query.id).await.unwrap();
    let to = if channel.is_enabled {
        //关闭代理监听
        tcp_proxy_manager::shutdown_by_channel(query.id).await;
        false
    } else {
        if nps::CLIENT_NPS_MAP
            .lock()
            .await
            .contains_key(&channel.client_id)
        {
            //如果当前客户端在线
            tcp_proxy_manager::accept_channel(channel).await.unwrap(); //开启隧道监听
        }
        true
    };
    channel_dao::toggle_enable(&conn, query.id, to)
        .await
        .unwrap();
}

// // 表单验证
// func validate(inForm form.ChannelEditForm) error {
// 	if len(inForm.Name) == 0 {
// 		return &controller.BusinessException{
// 			Message: "请填写隧道名",
// 		}
// 	}
// 	if len(inForm.Name) > 32 {
// 		return &controller.BusinessException{
// 			Message: "隧道名长度不能超过32个字符",
// 		}
// 	}
// 	//if len(inForm.ServerPort) == 0 {
// 	//	return &controller.BusinessException{
// 	//		Message: "服务端口必须设置",
// 	//	}
// 	//}
// 	//port, err := strconv.ParseInt(inForm.ServerPort, 10, 64)
// 	//if err != nil {
// 	//	return &controller.BusinessException{
// 	//		Message: "服务端口必须是一个数字",
// 	//	}
// 	//}
// 	if inForm.ServerPort < 0 || inForm.ServerPort > 65535 {
// 		return &controller.BusinessException{
// 			Message: "服务端口必须在0到65535之间",
// 		}
// 	}
// 	portChannel := ChannelDao.SelectByPort(inForm.ServerPort)
// 	if inForm.Id == 0 { //创建时
// 		if portChannel != nil {
// 			return &controller.BusinessException{
// 				Message: fmt.Sprintf("端口:%d已经被其他隧道占用", inForm.ServerPort),
// 			}
// 		}
// 	} else {
// 		if portChannel != nil && portChannel.Id != inForm.Id {
// 			return &controller.BusinessException{
// 				Message: fmt.Sprintf("端口:%d已经被其他隧道占用", inForm.ServerPort),
// 			}
// 		}
// 	}
// 	portForward := ForwardDao.SelectByPort(inForm.ServerPort)
// 	if portForward != nil {
// 		return &controller.BusinessException{
// 			Message: fmt.Sprintf("端口:%d已被端口转发:%s 占用", portForward.Port, portForward.Name),
// 		}
// 	}
// 	return nil
// }

mod model {
    use serde::{Deserialize, Serialize};
    use validator::Validate;

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

        // 乐观排他用的版本号
        pub version: i64,
    }

    #[derive(Debug, Default, Serialize, Deserialize, Validate)]
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
        pub remark: Option<String>,

        // 乐观排他用的版本号
        pub version: i64,
    }
}
