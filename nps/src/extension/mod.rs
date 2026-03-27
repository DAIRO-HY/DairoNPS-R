pub mod number;

use crate::web::model::ResultData;
use axum::{
    Json,
    body::Body,
    extract::Form,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use maplit::hashmap;
use rusqlite::types::FromSql;
use rusqlite::{Connection, Params};
use serde_json::json;
use std::{collections::HashMap, fmt::format};

// 这个模块提供了一个扩展 trait，用于简化查询单个值的操作
pub trait SelectSingleExt {
    fn select_single<T, P>(&self, sql: &str, params: P) -> T
    where
        T: FromSql,
        P: Params;
}
impl SelectSingleExt for Connection {
    fn select_single<T, P>(&self, sql: &str, params: P) -> T
    where
        T: FromSql,
        P: Params,
    {
        self.query_row(sql, params, |row| row.get::<_, T>(0))
            .unwrap()
    }
}

// 这个模块提供了一个扩展 trait，用于简化查询单个值的操作
pub trait ResponseEmptyExt {
    fn empty() -> Response;
    fn text(msg: String) -> Response;
    fn json<T: serde::Serialize>(msg: T) -> Response;
    fn field_error(field: &str, msg: &str) -> Response;
    fn field_errors(errors: validator::ValidationErrors) -> Response;
}
impl ResponseEmptyExt for Response {
    /// 这个方法用于返回一个空响应，状态码为200
    fn empty() -> Response {
        // Response::builder()
        //     .status(200)
        //     .body(axum::body::Body::empty())
        //     .unwrap()
        (StatusCode::OK, "").into_response()
    }

    /// 这个方法用于返回一个文本响应，状态码为200
    fn text(msg: String) -> Response {
        (StatusCode::OK, msg).into_response()
    }

    /// 这个方法用于返回一个JSON响应，状态码为200
    fn json<T: serde::Serialize>(data: T) -> Response {
        (StatusCode::OK, Json(data)).into_response()
    }

    /// 这个方法用于返回一个字段错误响应，状态码为500，包含一个字段和错误信息
    fn field_error(field: &str, msg: &str) -> Response {
        let result = ResultData {
            code: 2,
            msg: "表单验证失败".to_string(),
            data: hashmap! {field => vec![msg]},
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(result)).into_response()
    }

    /// 将验证错误转换为响应
    fn field_errors(errors: validator::ValidationErrors) -> Response {
        let field_error_map: HashMap<String, Vec<String>> = errors
            .field_errors()
            .into_iter()
            .map(|it| {
                (
                    it.0.to_string(),
                    it.1.iter()
                        .filter_map(|err| {
                            if let Some(message) = &err.message {
                                Some(message.to_string())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        let result = ResultData {
            code: 2,
            msg: "表单验证失败".to_string(),
            data: field_error_map,
        };
        (StatusCode::INTERNAL_SERVER_ERROR, Json(result)).into_response()
    }
}
