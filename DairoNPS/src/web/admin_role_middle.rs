use crate::dao::{admin_dao, permission_dao_ext, role_dao};
use crate::web::admin_auth_middle::AdminAuthInfo;
use axum::extract::{OriginalUri, Request};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

// 角色id对应的权限路径
static ROLE_ID2_PERMISSION_PATHS: LazyLock<RwLock<HashMap<i64, String>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

// 所有需要验证权限的路径
static ALL_PERMISSION_STR: LazyLock<String> = LazyLock::new(|| {
    let permission = permission_dao_ext::select_web_path().join(",");
    format!(",{},", permission)
});

pub async fn init() {
    let mut ctx = lib_db::get_context();
    let roles = role_dao::select_active(&mut ctx).await.unwrap();

    let mut map = ROLE_ID2_PERMISSION_PATHS.write().unwrap();
    map.clear();
    roles.into_iter().for_each(|it| {
        map.insert(it.id, it.permission);
    });
    drop(map);
}

/// 是否管理员验证
pub async fn auth(
    auth: AdminAuthInfo,
    OriginalUri(original_uri): OriginalUri,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let mut ctx = lib_db::get_context();
    let admin = admin_dao::select_one(&mut ctx, auth.login_id)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    //得到当前角色权限路径字符串
    let role_permission_str = ROLE_ID2_PERMISSION_PATHS
        .read()
        .unwrap()
        .get(&admin.roleId)
        .cloned()
        .unwrap_or_default();

    //得到需要验证权限部分路径 (使用原始路径, 因为 nest 会剥离 /admin 前缀)
    let current_permission_path = first_n_segments(original_uri.path(), 2);
    let current_permission_path = &format!(",{},", current_permission_path);

    //判断当前URL路径是否需要验证权限
    if !ALL_PERMISSION_STR.contains(current_permission_path) {
        return Ok(next.run(req).await);
    }

    if format!(",{},", role_permission_str).contains(current_permission_path) {
        return Ok(next.run(req).await);
    }
    Err(StatusCode::FORBIDDEN)
}

fn first_n_segments(path: &str, n: usize) -> &str {
    let bytes = path.as_bytes();
    let mut count = 0;

    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'?' | b'#' => return &path[..i], // 遇到查询串/锚点提前结束
            b'/' => {
                count += 1;
                if count == n + 1 {
                    return &path[..i];
                }
            }
            _ => {}
        }
    }
    path
}
