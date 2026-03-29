use crate::dao::{channel_dao, channel_data_dao};
use crate::dao::channel_dao::Channel;
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
use std::collections::HashMap;
use validator::Validate;
use crate::nps::nps_proxy::tcp_proxy_manager;

///  隧道列表
pub async fn list() -> Response {
    let conn = db_util::new_connection();

    let client_id_name_map = client_dao::select_all(&conn)
        .unwrap_or_default()
        .into_iter()
        .rev()
        .map(|it| (it.id, it.name))
        .collect::<HashMap<_, _>>();
    let list = channel_dao::select_all(&conn)
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
            enable_state: it.enable_state,
            in_data: it.in_data.data_size(),
            out_data: it.out_data.data_size(),
            security_state: it.security_state,
            error: it.error,
        })
        .collect::<Vec<_>>();
    Json(list).into_response()
}

/// 隧道详情获取API
pub async fn detail(AppQuery(query): AppQuery<model::DetailQuery>) -> Response {
    let conn = db_util::new_connection();
    let detail = if query.id > 0 {
        let Ok(channel) = channel_dao::select_one(&conn, query.id) else {
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
            enable_state: if channel.enable_state == 0 {
                "关闭"
            } else {
                "开启"
            }
            .to_string(),
            in_data: channel.in_data.data_size(),
            out_data: channel.out_data.data_size(),
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
    return Json(detail).into_response();

    // client := ClientDao.SelectOne(ClientId)
    // var outForm form.ChannelEditForm
    // if Id == 0 {
    // 	outForm = form.ChannelEditForm{
    // 		Mode: 1,
    // 	}
    // } else { //修改时
    // 	channelDto := ChannelDao.SelectOne(Id)
    // 	outForm = form.ChannelEditForm{
    // 		Id:            channelDto.Id,
    // 		Name:          channelDto.Name,
    // 		Mode:          channelDto.Mode,
    // 		Remark:        channelDto.Remark,
    // 		ServerPort:    channelDto.ServerPort,
    // 		TargetPort:    channelDto.TargetPort,
    // 		Date:          Date.FormatByTimespan(channelDto.Date),
    // 		EnableState:   Bool.Is(channelDto.EnableState == 0, "关闭", "开启"),
    // 		InData:        Number.ToDataSize(channelDto.InData),
    // 		OutData:       Number.ToDataSize(channelDto.OutData),
    // 		SecurityState: channelDto.SecurityState,
    // 	}
    // }
    // outForm.ClientId = ClientId
    // outForm.ClientName = client.Name
    // return outForm
}

// Edit 提交表单API
// post:/channel_list/channel_edit/edit
pub async fn edit(AppForm(form): AppForm<model::ChannelEdit>) -> Response {
    if let Err(e) = form.validate() {
        //验证表单数据是否合法
        return Response::field_errors(e);
    }

    let conn = db_util::new_connection();
    let mut channel = if form.id == 0 {
        Channel{
            enable_state:1,
            ..Default::default()
        }
    } else {
        if let Ok(it) = channel_dao::select_one(&conn, form.id) {
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
    if channel.id == 0 {
        if let Err(e) = channel_dao::insert(&conn, channel) {
            err = Some(e);
        }
    } else {
        err = channel_dao::update(&conn, channel);
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
    crate::application::restart_mark();//标记需要重启
    Response::empty()
}

/// 通过id删除一个隧道
pub async fn delete(AppQuery(query): AppQuery<IdQuery>) -> Response {
    let mut conn = db_util::new_connection();

    let Ok(tx) = conn.transaction() else{
        return biz_error!("开启事务失败");
    };
    if let Some(e) = channel_dao::purge(&tx, query.id){
        return biz_errorf!("删除失败:{}", e);
    }
    // channel_data_dao::delete_by_channel_id(&tx, query.id).unwrap();

    //提交事务
    let _ = tx.commit();
    // drop(conn);

    // //关闭代理监听
    // udp_proxy.ShutdownByChannel(channel.Id)
    tcp_proxy_manager::shutdown_by_channel(query.id).await;
    // crate::application::restart_mark();//标记需要重启
    Response::empty()
}

/// 修改可用状态
pub async fn toggle_enable(AppQuery(query): AppQuery<IdQuery>) {
    let conn = db_util::new_connection();
	let channel = channel_dao::select_one(&conn, query.id).unwrap();
	let to = if channel.enable_state == 0 {
        if crate::nps::CLIENT_SESSION.lock().await.contains_key(&channel.client_id){//如果当前客户端在线
            tcp_proxy_manager::accept_channel(channel).await;//开启隧道监听
        }
        1
		// clientDto := ClientDao.SelectOne(channel.ClientId)
		// if tcp_client.IsOnline(clientDto.Id) {
		// 	udp_proxy.AcceptClient(clientDto) //重新开启监听该客户端
		// }
	} else {

        // //关闭代理监听
        // udp_proxy.ShutdownByChannel(channel.Id)
        tcp_proxy_manager::shutdown_by_channel(query.id).await;
        0
	};
    channel_dao::toggle_enable(&conn, query.id, to);
    // crate::application::restart_mark();//标记需要重启
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
        pub mode: i8,
        pub server_port: i16,
        pub target_port: String,
        pub enable_state: i8,
        pub in_data: String,
        pub out_data: String,
        pub security_state: i8,
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
        pub mode: i8,
        pub server_port: i16,
        pub target_port: String,
        pub remark: Option<String>,
        pub created_at: String,
        pub enable_state: String,
        pub in_data: String,
        pub out_data: String,
        pub security_state: i8,

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
        pub mode: i8,
        pub server_port: i16,
        pub target_port: String,
        pub security_state: i8,
        pub remark: Option<String>,

        // 乐观排他用的版本号
        pub version: i64,
    }
}
