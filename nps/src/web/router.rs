use crate::extension::ResponseEmptyExt;
use axum::{
    Router,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use std::env;

/// 导入set_html_routes函数
include!(concat!(env!("OUT_DIR"), "/resource_routes.rs.block"));

pub fn ready() {
    tokio::spawn(init_router());
}

async fn init_router() {
    let app = set_html_routes(Router::new())
        .route("/", get(super::index::index))
        .route(
            "/test",
            get(async || -> Response {
                let logged_info = crate::web::controller::login::LOGGED_INFO.read().await;
                Response::ok(logged_info.token.clone())
            }),
        )
        .route("/login/do_login", post(super::controller::login::do_login))
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
        .route("/hello", get(hello));

    // run our app with hyper, listening globally on port 1880
    let listener = tokio::net::TcpListener::bind("0.0.0.0:1880").await.unwrap();
    axum::serve(listener, app).await;
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
