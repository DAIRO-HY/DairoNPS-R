use axum::{
    extract::Form,
    routing::post,
    Router,
};


pub async fn index() -> &'static str {
    "Hello, Index!"
}