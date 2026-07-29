use crate::util::image_util;
use crate::util::image_util::DynamicImageExt;
use crate::web::{admin_auth_middle, admin_role_middle, app_auth_middle};
use crate::{application, web};
use axum::error_handling::HandleErrorLayer;
use axum::extract::{DefaultBodyLimit, Request};
use axum::middleware::Next;
use axum::{
    BoxError, Json, Router,
    http::{StatusCode, header},
    middleware,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use axum_extra::extract::CookieJar;
use lib_axum_extract::{AppJson, AppPath, AppQuery};
use lib_np_common::time_util;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs};
use tower::ServiceBuilder;
use tower::timeout::error::Elapsed;

/// 导入set_html_routes函数
include!(concat!(env!("OUT_DIR"), "/static_routes.rs"));

/// 从查询参数中提取id
#[derive(Deserialize, Serialize, Debug)]
pub struct IdQuery {
    /// 当参数没有传时,使用默认值
    #[serde(default)]
    pub id: i64,
}

/// 从查询参数中获取一个参数
#[derive(Deserialize, Serialize, Debug)]
pub struct SingleQuery<T> {
    pub value: T,
}

pub fn ready() {
    tokio::spawn(init_router());
}

async fn init_router() {
    admin_role_middle::init().await;
    let app = add_static_routes(Router::new())
        .route("/", get(super::controller::index::index))
        .route("/do_collect", get(do_collect))
        .route("/keep_live_on", get(keep_live_on))
        .route("/keep_live_off", get(keep_live_on_off))
        .route("/image_resize", get(image_resize))
        .route(
            "/api/login/do_login",
            post(super::controller::login::do_login),
        )
        .route("/api/login/forget", get(super::controller::login::forget))
        .route("/api/common/global", get(super::controller::common::global))
        .nest(
            "/api",
            Router::new()
                .route("/login/logout", delete(super::controller::login::logout))
                .route("/common/restart", put(super::controller::common::restart))
                .route(
                    "/common/dropdown/{tag}",
                    get(super::controller::common::dropdown),
                )
                .route("/chart/real_len", get(super::controller::chart::real_len))
                .route("/chart/data_len", get(super::controller::chart::data_len))
                // .route(
                //     "/index/nps_status",
                //     get(super::controller::index::get_nps_status),
                // )
                .route("/client/list", get(super::controller::client::list))
                .route("/client/detail", get(super::controller::client::detail))
                .route("/client/edit", post(super::controller::client::edit))
                .route("/client/delete", delete(super::controller::client::delete))
                .route(
                    "/client/toggle_enable",
                    put(super::controller::client::toggle_enable),
                )
                .route("/channel/list", get(super::controller::channel::list))
                .route("/channel/detail", get(super::controller::channel::detail))
                .route("/channel/edit", post(super::controller::channel::edit))
                .route(
                    "/channel/delete",
                    delete(super::controller::channel::delete),
                )
                .route(
                    "/channel/toggle_enable",
                    put(super::controller::channel::toggle_enable),
                )
                .route("/forward/list", get(super::controller::forward::list))
                .route("/forward/detail", get(super::controller::forward::detail))
                .route("/forward/edit", post(super::controller::forward::edit))
                .route(
                    "/forward/delete",
                    delete(super::controller::forward::delete),
                )
                .route(
                    "/forward/toggle_enable",
                    put(super::controller::forward::toggle_enable),
                )
                .route("/bridge/list", get(super::controller::bridge::list))
                .route("/bridge/broken", put(super::controller::bridge::broken))
                // .route("/", get(async || -> Response {
                //     static_file_res("content", "mime.as_ref().to_string()")
                //     // // 你可以按需增加更多头：Cache-Control / ETag / Last-Modified 等
                //     // // 下面演示一个简单的缓存头（可按需调整策略）
                //     // let headers = [
                //     //     (header::CONTENT_TYPE, "mime.as_ref().to_string()".to_string()),
                //     //     (
                //     //         header::CACHE_CONTROL,
                //     //         "public, max-age=86400, immutable".to_string(),// 1 天缓存，且内容不变（适合版本化资源）
                //     //     ),
                //     // ];
                //     // (StatusCode::OK, headers, "content".to_string()).into_response()
                //  }))
                // .route("/static/{*path}", get(super::static_file::handler))
                .route("/query", get(query_test))
                .route("/path/{id}", get(path_test))
                .route("/json", post(json))
                .layer(middleware::from_fn(auth_login)),
        )
        .nest(
            "/common",
            Router::new()
                .route("/role_dropdown", get(super::admin::common::role_dropdown))
                .route(
                    "/subsidy_type_dropdown",
                    get(super::admin::common::subsidy_type_dropdown),
                )
                .route(
                    "/subsidy_state_dropdown",
                    get(super::admin::common::subsidy_state_dropdown),
                )
                .route(
                    "/farming_type_dropdown/{id}",
                    get(super::admin::common::farming_type_dropdown),
                )
                .route(
                    "/address_dropdown_by_parent_id/{id}",
                    get(super::admin::common::address_dropdown_by_parent_id),
                )
                .route(
                    "/active_type_dropdown",
                    get(super::admin::common::active_type_dropdown),
                )
                .route(
                    "/household_dropdown",
                    get(super::admin::common::household_dropdown),
                )
                .route(
                    "/breeder_dropdown",
                    get(super::admin::common::breeder_dropdown),
                ),
        )
        .nest(
            "/app",
            Router::new()
                .route("/login/do_login", post(super::app::login::do_login))
                .route(
                    "/login/wx_code2openid/{code}",
                    get(super::app::login::wx_code2openid),
                )
                .route(
                    "/login/wx_login/{openid}",
                    post(super::app::login::wx_login),
                ),
        )
        .nest(
            "/app",
            Router::new()
                .route("/image", get(super::image::show))
                .route("/image/upload", post(super::image::upload))
                .route(
                    "/common/room_dropdown",
                    get(super::admin::common::room_dropdown),
                )
                .route("/login/logout", delete(super::app::login::logout))
                .route("/user/info", get(super::app::user::info))
                .route(
                    "/user/read_id_card_face",
                    get(super::app::user::read_id_card_face),
                )
                .route(
                    "/user/read_id_card_back",
                    get(super::app::user::read_id_card_back),
                )
                .route("/user/read_sbk_card", get(super::app::user::read_sbk_card))
                .route("/user/edit", put(super::app::user::edit))
                .route("/user/bind_wx/{openid}", put(super::app::user::bind_wx))
                .route("/user/unbind_wx/{openid}", put(super::app::user::unbind_wx))
                .route("/user/modify_pwd", put(super::app::user::modify_pwd))
                .route(
                    "/user/bind_phone_by_wx_code/{code}",
                    put(super::app::user::bind_phone_by_wx_code),
                )
                .route(
                    "/user/get_phone_by_wx_code/{code}",
                    get(super::app::user::get_phone_by_wx_code),
                )
                .route("/farming/list", get(super::app::farming::list))
                .route("/farming/info/{id}", get(super::app::farming::info))
                .route("/farming/edit", post(super::app::farming::edit))
                .route("/subsidy/list", get(super::app::subsidy::list))
                .route("/subsidy/info", get(super::app::subsidy::info))
                .route("/subsidy/apply", put(super::app::subsidy::apply))
                .route("/subsidy/cancel/{id}", put(super::app::subsidy::cancel))
                .route("/room/list", get(super::app::room::list))
                .route("/room/info/{id}", get(super::app::room::info))
                .route("/room/edit", post(super::app::room::edit))
                .route("/room/delete/{id}", delete(super::app::room::delete))
                .route(
                    "/active/type_dropdown",
                    get(super::app::active::type_dropdown),
                )
                .route(
                    "/active/vaccine_dropdown/{farming_ids}",
                    get(super::app::active::vaccine_dropdown),
                )
                .route(
                    "/active/food_dropdown/{farming_ids}",
                    get(super::app::active::food_dropdown),
                )
                .route(
                    "/active/sick_dropdown/{farming_ids}",
                    get(super::app::active::sick_dropdown),
                )
                .route("/active/add", post(super::app::active::add))
                .route("/active/list/{farming_id}", get(super::app::active::list))
                .layer(middleware::from_fn(app_auth_middle::auth)),
        )
        .nest(
            "/admin",
            Router::new()
                .route("/login/init", post(super::admin::login::init))
                .route("/login/do_login", post(super::admin::login::do_login)),
        )
        .nest(
            "/admin",
            Router::new()
                .route("/image", get(super::image::show))
                .route("/image/upload", post(super::image::upload))
                .route("/login/logout", delete(super::admin::login::logout))
                .route("/mine/info", get(super::admin::mine::info))
                .route("/mine/modify_pwd", put(super::admin::mine::modify_pwd))
                .route("/admin/init/{id}", get(super::admin::admin::init))
                .route("/admin/info/{id}", get(super::admin::admin::info))
                .route("/admin/delete/{id}", delete(super::admin::admin::delete))
                .route("/admin/edit", post(super::admin::admin::edit))
                .route("/admin/list", get(super::admin::admin::list))
                .route("/active_log/list", get(super::admin::active_log::list))
                .route("/farming/list", get(super::admin::farming::list))
                .route("/farming/info/{id}", get(super::admin::farming::info))
                .route("/farming/edit", put(super::admin::farming::edit))
                .route(
                    "/farming/delete/{id}",
                    delete(super::admin::farming::delete),
                )
                .route(
                    "/farming/batch_delete/{ids}",
                    delete(super::admin::farming::batch_delete),
                )
                .route("/subsidy/list", get(super::admin::subsidy::list))
                .route(
                    "/subsidy/scan_item/{fid}",
                    get(super::admin::subsidy::scan_item),
                )
                .route("/subsidy/info/{id}", get(super::admin::subsidy::info))
                .route("/subsidy/accept1", put(super::admin::subsidy::accept1))
                .route("/subsidy/reject1", put(super::admin::subsidy::reject1))
                .route("/subsidy/accept2", put(super::admin::subsidy::accept2))
                .route("/subsidy/reject2", put(super::admin::subsidy::reject2))
                .route("/subsidy/accept3", put(super::admin::subsidy::accept3))
                .route("/subsidy/reject3", put(super::admin::subsidy::reject3))
                .route(
                    "/subsidy/reapply_to_wait",
                    put(super::admin::subsidy::reapply_to_wait),
                )
                .route(
                    "/subsidy/reapply_to_accept1",
                    put(super::admin::subsidy::reapply_to_accept1),
                )
                .route(
                    "/subsidy/reapply_to_accept2",
                    put(super::admin::subsidy::reapply_to_accept2),
                )
                // .route(
                //     "/subsidy/modify_breeder",
                //     put(super::admin::subsidy::modify_breeder),
                // )
                .route(
                    "/subsidy/modify_money",
                    put(super::admin::subsidy::modify_money),
                )
                .route(
                    "/subsidy/delete/{id}",
                    delete(super::admin::subsidy::delete),
                )
                // .route("/subsidy/batch_delete/{ids}", delete(super::admin::subsidy::batch_delete))
                .route("/user/list", get(super::admin::user::list))
                .route("/user/edit", post(super::admin::user::edit))
                .route("/user/info/{id}", get(super::admin::user::info))
                .route("/user/delete/{id}", delete(super::admin::user::delete))
                .route(
                    "/user/batch_delete/{ids}",
                    delete(super::admin::user::batch_delete),
                )
                .route(
                    "/user/batch_cancel_delete/{ids}",
                    put(super::admin::user::batch_cancel_delete),
                )
                .route("/address/info/{id}", get(super::admin::address::info))
                .route("/address/edit", post(super::admin::address::edit))
                .route(
                    "/address/delete/{id}",
                    delete(super::admin::address::delete),
                )
                .route("/fid/list", get(super::admin::fid::list))
                .route("/fid/info/{id}", get(super::admin::fid::info))
                .route("/fid/edit", post(super::admin::fid::edit))
                .route("/fid/delete/{id}", delete(super::admin::fid::delete))
                .route("/farming_type/list", get(super::admin::farming_type::list))
                .route(
                    "/farming_type/info/{id}",
                    get(super::admin::farming_type::info),
                )
                .route(
                    "/farming_type/dropdown",
                    get(super::admin::farming_type::dropdown),
                )
                .route("/role/edit", post(super::admin::role::edit))
                .route("/role/list", get(super::admin::role::list))
                .route("/role/info/{id}", get(super::admin::role::info))
                .route("/role/dropdown", get(super::admin::role::dropdown))
                .route("/role/delete/{id}", delete(super::admin::role::delete))
                .route(
                    "/system_config/edit",
                    post(super::admin::system_config::edit),
                )
                .route(
                    "/system_config/info",
                    get(super::admin::system_config::info),
                )
                .route(
                    "/admin_message/list",
                    get(super::admin::admin_message::list),
                )
                .route(
                    "/admin_message/count",
                    get(super::admin::admin_message::count),
                )
                .route(
                    "/admin_message/delete/{id}",
                    delete(super::admin::admin_message::delete),
                )
                .route(
                    "/admin_message/read/{id}",
                    put(super::admin::admin_message::read),
                )
                .route(
                    "/admin_message/read_all",
                    put(super::admin::admin_message::read_all),
                )
                .layer(middleware::from_fn(admin_role_middle::auth))
                .layer(middleware::from_fn(admin_auth_middle::auth)),
        )
        //限制文件上传大小
        .layer(DefaultBodyLimit::max(20000 * 1024 * 1024))
        .layer(
            ServiceBuilder::new()
                // `timeout` will produce an error if the handler takes
                // too long so we must handle those
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .timeout(Duration::from_secs(300)),
        );
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", application::ARGS.web_port))
        .await
        .unwrap();
    let _ = axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            application::SHUTDOWN_NOTIFY.notified().await;

            // 标记axum已经退出
            application::IS_AXUM_DROP.store(true, Ordering::Relaxed);
        })
        .await;
}

async fn handle_timeout_error(err: BoxError) -> (StatusCode, String) {
    if err.is::<Elapsed>() {
        (
            StatusCode::REQUEST_TIMEOUT,
            "Request took too long".to_string(),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {err}"),
        )
    }
}

/// 验证是否已经登录
pub async fn auth_login(jar: CookieJar, req: Request, next: Next) -> Result<Response, StatusCode> {
    // 从 Cookie 获取 session / token
    let Some(cookie) = jar.get("token") else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let token = cookie.value();
    if token != web::API_TOKEN.load().to_string() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let now = time_util::current_millis();

    //验证已经登录时间
    if now - web::LOGIN_TIME.load(Ordering::Relaxed) > web::MAX_LOGIN_TIME {
        return Err(StatusCode::UNAUTHORIZED);
    }

    //验证Session时间
    if now - web::LAST_USE_TIME.load(Ordering::Relaxed) > web::LOGIN_SESSION_TIME {
        return Err(StatusCode::UNAUTHORIZED);
    }

    //更新最后把使用时间
    web::LAST_USE_TIME.store(now, Ordering::Relaxed);
    Ok(next.run(req).await)
}

/// 处理静态文件请求
pub async fn handler(content: &str) -> Response {
    // let content = "std::str::from_utf8(res_embed.data.as_ref())".to_string();

    // 你可以按需增加更多头：Cache-Control / ETag / Last-Modified 等
    // 下面演示一个简单的缓存头（可按需调整策略）
    let headers = [
        (
            header::CONTENT_TYPE,
            "mime.as_ref().to_string()".to_string(),
        ),
        (
            header::CACHE_CONTROL,
            "public, max-age=86400, immutable".to_string(), // 1 天缓存，且内容不变（适合版本化资源）
        ),
    ];
    (StatusCode::OK, headers, content.to_string()).into_response()
}

async fn do_collect() -> &'static str {
    unsafe {
        crate::mi_collect(true);
    }
    "mi_collect_success"
}

async fn keep_live_on() -> &'static str {
    // tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    "Hello, World!"
}

async fn keep_live_on_off() -> impl IntoResponse {
    ([(header::CONNECTION, "close")], "hello")
}

async fn image_resize() -> &'static str {
    image_util::resize(
        Path::new("/Users/zhoulq/dev/java/idea/FarmingServer/test2/input.jpg"),
        300,
    )
    .unwrap()
    .jpeg(
        Path::new("/Users/zhoulq/dev/java/idea/FarmingServer/test2/output_resize.jpg"),
        80,
    )
    .unwrap();
    image_util::crop(
        Path::new("/Users/zhoulq/dev/java/idea/FarmingServer/test2/input.jpg"),
        300,
        300,
    )
    .unwrap()
    .jpeg(
        Path::new("/Users/zhoulq/dev/java/idea/FarmingServer/test2/output_crop.jpg"),
        80,
    )
    .unwrap();
    "图片处理完成"
}

#[derive(Deserialize, Serialize, Debug)]
pub struct QueryParams {
    pub name: String,
    pub age: u32,
}
async fn query_test(AppQuery(q): AppQuery<QueryParams>) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(q)).into_response()
}

async fn json(AppJson(q): AppJson<QueryParams>) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(q)).into_response()
}
async fn path_test(AppPath(id): AppPath<i64>) -> Response {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", id)).into_response()
}
