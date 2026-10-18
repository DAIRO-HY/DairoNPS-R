use crate::web;
use crate::web::biz_error::BizError;
use hyperoute_core::{Next, Request, Response};
use hyperoute_macro::FILTER;
use lib_np_common::time_util;
use std::sync::atomic::Ordering;

/// 验证是否已经登录
#[FILTER(
    order = 10,
    include = ["/api/**"],
    exclude = ["/api/login/do_login", "/api/login/forget", "/api/common/global"]
)]
pub async fn auth_login(req: Request, next: Next) -> Result<Response, BizError> {
    // 从 Cookie 获取 session / token
    let Some(token) = get_cookie(&req, "token") else {
        return Err(BizError::unauthorized());
    };
    if token != web::API_TOKEN.load().as_str() {
        return Err(BizError::unauthorized());
    }
    let now = time_util::current_millis();

    //验证已经登录时间
    if now - web::LOGIN_TIME.load(Ordering::Relaxed) > web::MAX_LOGIN_TIME {
        return Err(BizError::unauthorized());
    }

    //验证Session时间
    if now - web::LAST_USE_TIME.load(Ordering::Relaxed) > web::LOGIN_SESSION_TIME {
        return Err(BizError::unauthorized());
    }

    //更新最后把使用时间
    web::LAST_USE_TIME.store(now, Ordering::Relaxed);
    Ok(next.run(req).await)
}

/// 从请求头中获取指定名称的Cookie值
fn get_cookie<'a>(req: &'a Request, name: &str) -> Option<&'a str> {
    req.headers()
        .get_all("cookie")
        .iter()
        .filter_map(|it| it.to_str().ok())
        .flat_map(|it| it.split(';'))
        .filter_map(|it| it.trim().split_once('='))
        .find(|(key, _)| *key == name)
        .map(|(_, value)| value)
}
