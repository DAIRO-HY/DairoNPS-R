use crate::constant::nps_constant;
use crate::extension::ResponseEmptyExt;
use crate::{biz_error, biz_errorf};
use axum::{
    Json, Router,
    extract::Form,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use once_cell::sync::Lazy;
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU8, AtomicU64, Ordering};
use tokio::sync::RwLock;
use validator::{Validate, ValidationErrors};
use crate::web::extract::{AppForm, AppQuery};


pub static LOGGED_INFO: Lazy<RwLock<LoggedInfo>> = Lazy::new(|| {
    RwLock::new(LoggedInfo {
        token: String::new(),
        last_time: 0,
    })
});

// 记录密码错误次数
pub static LOGIN_ERROR_COUNT: AtomicU8 = AtomicU8::new(0);

// DoLogin 登录API
// post:/login/do_login
pub async fn do_login(AppForm(form): AppForm<LoginForm>) -> Response {
    if LOGIN_ERROR_COUNT.load(Ordering::Relaxed) > 10 {
        return biz_error!("用户名或密码错误次数超过限制，请联系管理员。");
    }
    if let Err(e) = form.validate() {//验证表单数据是否合法
        return Response::field_errors(e);
    }
    if form.name != nps_constant::LOGIN_NAME || form.pwd != nps_constant::LOGIN_PWD {
        LOGIN_ERROR_COUNT.fetch_add(1, Ordering::Relaxed);
        return biz_error!("用户名或密码错误");
    }

    // 登录成功，重置错误次数
    LOGIN_ERROR_COUNT.store(0, Ordering::Relaxed);

    // timeRand := time.Now().UnixMilli() + int64(rand.IntN(900000)+100000)
    // timeRandStr := strconv.FormatInt(timeRand, 10)
    // token := String.ToMd5(timeRandStr)
    // tokenCookie := &http.Cookie{
    // 	Name:    login_state.COOKIE_TOKEN,
    // 	Value:   token,
    // 	Path:    "/",
    // 	Expires: time.Now().AddDate(100, 0, 0), //100年以后过期
    // 	MaxAge:  100 * 365 * 24 * 60 * 60,
    // 	//HttpOnly: true,
    // }
    // http.SetCookie(writer, tokenCookie)

    // login_state.Login(token);

    // 生成一个随机的32位token，作为登录状态的标识
    let token = Alphanumeric.sample_string(&mut rand::rng(), 32);

    let mut logged_info = LOGGED_INFO.write().await;
    logged_info.last_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    logged_info.token = token.clone();
    Response::empty()
}

// // Logout 退出登录
// // post:/login/login_out
// func Logout() {
// 	login_state.LoginOut()
// }

// // Logout 退出登录
// // post:/login/login_out/test
// func LogoutTest() string {
// 	return "123"
// }




/// 登录表单数据结构
#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct LoginForm {
    //管理员登录名
    #[validate(length(min = 1, message = "用户名不能为空"))]
    name: String,

    //管理员登录密码
    #[validate(length(min = 1, message = "密码不能为空"))]
    pwd: String,
}

/// 登录状态信息结构体
pub struct LoggedInfo {
    //登录状态token
    pub token: String,

    //登录时间戳，单位毫秒
    pub last_time: u128,
}