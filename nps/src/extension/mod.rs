use rusqlite::{Connection, Params};
use rusqlite::types::FromSql;
use axum::{
    extract::Form,
    http::{StatusCode, header},
    response::{IntoResponse, Response}
};

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
        self.query_row(sql, params, |row| row.get::<_, T>(0)).unwrap()
    }
}



// 这个模块提供了一个扩展 trait，用于简化查询单个值的操作
pub trait ResponseEmptyExt {
    fn empty() -> Response;
    fn ok(msg: String) -> Response;
}
impl ResponseEmptyExt for Response {
    fn empty() -> Response
    {
        Response::builder()
        .status(200)
        .body(axum::body::Body::empty())
        .unwrap()
    }
    fn ok(msg: String) -> Response
    {
        Response::builder()
        .status(200)
        .body(axum::body::Body::from(msg))
        .unwrap()
    }
}


