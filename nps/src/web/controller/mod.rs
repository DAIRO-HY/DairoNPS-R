pub mod login;
pub mod index;
pub mod client;
pub mod channel;
pub mod common;
pub mod chart;

use axum::{
    Json, Router,
    extract::Form,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
