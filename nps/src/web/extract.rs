use crate::web::model::ResultData;
use crate::{biz_error, biz_errorf, extension::ResponseEmptyExt};
use axum::{
    Form, Json,
    body::Body,
    extract::{
        FromRequest, FromRequestParts, Path, Query, Request,
        rejection::{FormRejection, JsonRejection, PathRejection, QueryRejection},
    },
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use maplit::hashmap;
use std::usize;

/// 这个模块定义了三个新的提取器：AppQuery、AppForm 和 AppJson，分别用于处理查询参数、表单数据和 JSON 数据的提取，并且在提取失败时返回一个统一格式的错误响应。
pub struct AppQuery<T>(pub T);
impl<S, T> FromRequestParts<S> for AppQuery<T>
where
    Query<T>: FromRequestParts<S, Rejection = QueryRejection>,
    S: Send + Sync,
{
    type Rejection = Response;
    async fn from_request_parts(ps: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match Query::<T>::from_request_parts(ps, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let rejection = rejection.to_string();
                error_handler(rejection)
            }
        }
    }
}
pub struct AppPath<T>(pub T);
impl<S, T> FromRequestParts<S> for AppPath<T>
where
    Path<T>: FromRequestParts<S, Rejection = PathRejection>,
    S: Send + Sync,
{
    type Rejection = Response;
    async fn from_request_parts(ps: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match Path::<T>::from_request_parts(ps, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let rejection = rejection.to_string();
                if !rejection.starts_with("Cannot parse") {
                    // 只有当路径参数无法解析时才返回错误，其他情况（如路径参数缺失）交给后续的路由匹配处理
                    return Err(biz_errorf!("请求数据格式错误: {}", rejection));
                }
                let mut it = rejection
                    .bytes()
                    .enumerate()
                    .filter(|&(_, b)| b == b'`')
                    .map(|(i, _)| i);

                let first = it.next().unwrap_or_default();
                let second = it.next().unwrap_or_default();
                let third = it.next().unwrap_or_default();
                let fourth = it.next().unwrap_or_default();
                if fourth == usize::default() {
                    // 这种情况说明路径参数缺失，交给后续的路由匹配处理
                    return Err(biz_errorf!("请求数据格式错误: {}", rejection));
                }

                //得到字段名
                let value = rejection[first + 1..second].trim();
                let ty = rejection[third + 1..fourth].trim();
                return Err(biz_errorf!(
                    "无法将路径参数的值 '{}' 转换为目标'{}' 类型",
                    value,
                    ty
                ));
            }
        }
    }
}

pub struct AppForm<T>(pub T);
impl<S, T> FromRequest<S> for AppForm<T>
where
    Form<T>: FromRequest<S, Rejection = FormRejection>,
    S: Send + Sync,
{
    type Rejection = Response;
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Form::<T>::from_request(req, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let rejection = rejection.to_string();
                return Err(biz_errorf!("请求数据格式错误: {}", rejection));
                error_handler(rejection)
            }
        }
    }
}

pub struct AppJson<T>(pub T);
impl<S, T> FromRequest<S> for AppJson<T>
where
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
    S: Send + Sync,
{
    type Rejection = Response;
    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        match Json::<T>::from_request(req, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let rejection = rejection.to_string();
                error_handler(rejection)
            }
        }
    }
}

/// 处理字段缺失或类型错误的情况，返回一个统一格式的错误响应
fn error_handler<T>(rejection: String) -> Result<T, axum::http::Response<Body>> {
    //Failed to deserialize the JSON body into the target type: missing field `age` at line 1 column 14
    //Failed to deserialize query string: missing field `age`
    //Failed to deserialize form body: missing field `pwd`
    if rejection.starts_with("Failed to deserialize query string: missing field")
        || rejection.starts_with("Failed to deserialize form body: missing field")
        || rejection
            .starts_with("Failed to deserialize the JSON body into the target type: missing field")
    {
        // 查询参数缺失
        not_found_error_handler(rejection)
    } else if rejection.starts_with("Failed to deserialize form body:")
        || rejection.starts_with("Failed to deserialize query string:")
        || rejection.starts_with("Failed to deserialize the JSON body into the target type:")
    {
        // 这三种错误都表示数据类型转换失败
        invalid_error_handler(rejection)
    } else {
        Err(biz_errorf!("请求数据格式错误: {}", rejection))
    }
}

/// 处理字段缺失，返回一个统一格式的错误响应
fn not_found_error_handler<T>(rejection: String) -> Result<T, axum::http::Response<Body>> {
    let start = rejection.find('`').unwrap_or(usize::MIN) + 1;
    let end = rejection.rfind('`').unwrap_or(usize::MIN);
    if end == usize::MIN {
        return Err(biz_errorf!("请求数据格式错误: {}", rejection));
    }

    //得到字段名
    let field = &rejection[start..end];
    let result = ResultData {
        code: 2,
        msg: "表单验证失败".to_string(),
        data: hashmap! {field => vec!["该字段必须"]},
    };
    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(result)).into_response())
}

/// 数据类型转换失败的错误处理器
fn invalid_error_handler<T>(rejection: String) -> Result<T, axum::http::Response<Body>> {
    // Failed to deserialize form body: name: invalid digit found in string
    // Failed to deserialize the JSON body into the target type: age: invalid type: string "30a", expected u32 at line 1 column 25
    // Failed to deserialize query string: age: invalid digit found in string

    // Failed to deserialize form body: name: cannot parse integer from empty string
    // Failed to deserialize query string: age: cannot parse integer from empty string
    // Failed to deserialize the JSON body into the target type: age: invalid type: string ""

    let mut it = rejection
        .bytes()
        .enumerate()
        .filter(|&(_, b)| b == b':')
        .map(|(i, _)| i);

    let first = it.next();
    let second = it.next();

    //得到字段名
    let field = if let (Some(a), Some(b)) = (first, second) {
        // +1 跳过冒号本身，再 trim 去掉空格
        rejection[a + 1..b].trim()
    } else {
        return Err(biz_errorf!("请求数据格式错误: {}", rejection));
    };
    let msg = if rejection.ends_with("invalid digit found in string") {
        "无效的数字格式"
    } else if rejection.ends_with("cannot parse integer from empty string") {
        "不能将空字符串转换为数字"
    } else {
        "类型转换失败"
    };

    let result = ResultData {
        code: 2,
        msg: "表单验证失败".to_string(),
        data: hashmap! {field => vec![msg]},
    };
    Err((StatusCode::INTERNAL_SERVER_ERROR, Json(result)).into_response())
}
