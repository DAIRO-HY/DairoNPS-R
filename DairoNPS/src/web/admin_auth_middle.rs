use crate::dao::{admin_token_dao, user_token_dao};
use axum::extract::Request;
use axum::http::{StatusCode, header};
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::CookieJar;
use axum_request_macros::FromExtension;

/// 验证是否已经登录
pub async fn auth(jar: CookieJar, mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let mut token = String::new();
    //优先从cookie中获取认证信息
    if let Some(cookie) = jar.get("token") {
        token = cookie.value().to_string();
    }
    if token.is_empty() {
        //再从authorization请求头获取认证信息
        if let Some(auth_head) = req.headers().get(header::AUTHORIZATION) {
            token = auth_head.to_str().unwrap_or_default().to_string();
        }
    }
    if token.is_empty() {
        //最后从query参数中获取
        if let Some(q) = req.uri().query() {
            token = form_urlencoded::parse(q.as_bytes())
                .find(|(k, _)| k == "_token")
                .map(|(_, v)| v.to_string())
                .unwrap_or_default();
        }
    }
    if token.is_empty() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    let mut ctx = lib_db::get_context();
    let admin_id = admin_token_dao::get_by_admin_id_by_token(&mut ctx, &token)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // 关键：塞进 extensions
    req.extensions_mut().insert(AdminAuthInfo {
        login_id: admin_id,
        token,
    });
    Ok(next.run(req).await)
}

/// 登录信息
#[derive(Clone, Debug, FromExtension)]
pub struct AdminAuthInfo {
    pub login_id: i64,
    pub token: String,
}
