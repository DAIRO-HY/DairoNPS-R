use crate::extension::ResponseEmptyExt;
use crate::{application, web};
use crate::web::extract::{AppJson, AppPath, AppQuery};
use axum::extract::Request;
use axum::middleware::Next;
use axum::routing::head;
use axum::{
    Json, Router,
    http::{StatusCode, header},
    middleware,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use axum_extra::extract::CookieJar;
use np_common::time_util;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::atomic::Ordering;

/// 导入set_html_routes函数
include!(concat!(env!("OUT_DIR"), "/resource_routes.rs.block"));

/// 从查询参数中提取id
#[derive(Deserialize, Serialize, Debug)]
pub struct IdQuery {
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
    let app = set_html_routes(Router::new())
        .route("/", get(super::controller::index::index))
        .route("/index/test", get(super::controller::index::test))
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
                .route(
                    "/index/nps_status",
                    get(super::controller::index::get_nps_status),
                )
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
                .route(
                    "/test",
                    get(async || -> Response { Response::text(web::API_TOKEN.load().to_string()) }),
                )
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
                .route("/hello", get(hello))
                .route("/query", get(query_test))
                .route("/path/{id}", get(path_test))
                .route("/json", post(json))
                .layer(middleware::from_fn(auth_login)),
        );
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}",application::ARGS.web_port)).await.unwrap();
    let _ = axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            application::SHUTDOWN_NOTIFY.notified().await;

            // 标记axum已经退出
            application::IS_AXUM_DROP.store(true, Ordering::Relaxed);
        })
        .await;
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

async fn hello() -> &'static str {
    "Hello, World!"
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
