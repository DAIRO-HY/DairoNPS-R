use std::sync::atomic::{AtomicU64, AtomicU8};
use std::sync::{Arc, LazyLock};
use arc_swap::ArcSwap;

pub mod router;
pub mod controller;
pub mod macros;
pub mod extract;
pub mod model;

/// 账号信息保存目录
pub const ACCOUNT_PATH: &str = "./account.txt";

/// 登录保留最大时间,过期强制重新登录
pub const MAX_LOGIN_TIME:u64 = 8 * 60 * 60 * 1000;

/// 回话保留最大时间(一段时间没有操作),过期强制重新登录
pub const LOGIN_SESSION_TIME:u64 = 15 * 60 * 1000;

/// 记录最后一次使用时间
pub static LAST_USE_TIME: AtomicU64 = AtomicU64::new(0);

/// 记录登录时间
pub static LOGIN_TIME: AtomicU64 = AtomicU64::new(0);

/// 存储登录TOKEN
pub static API_TOKEN: LazyLock<ArcSwap<String>> = LazyLock::new(|| {ArcSwap::new(Arc::new(String::new()))});

/// 记录密码错误次数
pub static LOGIN_ERROR_COUNT: AtomicU8 = AtomicU8::new(0);
