use crate::extension::string::StringExtension;
use crate::web::biz_error::BizError;
use crate::{biz_error, biz_errorf, web};
use hyperoute_core::Raw;
use hyperoute_macro::{DELETE, GET, POST};
use lib_np_common::time_util;
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use validator::Validate;

/// 登录API
#[POST("/api/login/do_login")]
pub async fn do_login(form: LoginForm) -> Result<Raw, BizError> {
    if web::LOGIN_ERROR_COUNT.load(Ordering::Relaxed) > 10 {
        return Err(biz_error!("用户名或密码错误次数超过限制，请联系管理员。"));
    }
    if let Err(e) = form.validate() {
        //验证表单数据是否合法
        return Err(BizError::fields(e));
    }
    let account_path = Path::new(web::ACCOUNT_PATH);
    if !fs::exists(account_path).unwrap() {
        //文件不存在
        let content = format!("{}\n{}", form.name, form.pwd.md5());
        fs::write(&account_path, content.as_bytes()).unwrap();
    }
    let account_account = fs::read_to_string(&account_path).unwrap();
    let account: Vec<&str> = account_account.split("\n").collect();
    if account.len() != 2 {
        return Err(biz_errorf!("文件:{}内容不合法,请删除后再重试!", web::ACCOUNT_PATH));
    }

    if form.name != account[0] || form.pwd.md5() != account[1] {
        web::LOGIN_ERROR_COUNT.fetch_add(1, Ordering::Relaxed);
        return Err(biz_error!("用户名或密码错误"));
    }

    // 登录成功，重置错误次数
    web::LOGIN_ERROR_COUNT.store(0, Ordering::Relaxed);

    // 生成一个随机的32位token，作为登录状态的标识
    let token = Alphanumeric.sample_string(&mut rand::rng(), 32);

    //记录登录信息,默认只允许一个客户端登录
    web::LOGIN_TIME.store(time_util::current_millis(), Ordering::Relaxed);
    web::LAST_USE_TIME.store(time_util::current_millis(), Ordering::Relaxed);
    web::API_TOKEN.store(Arc::new(token.clone()));

    // 写入 Cookie，全站生效，HttpOnly防止 JS 读取（安全）
    Ok(Raw::new("").header("set-cookie", format!("token={}; Path=/; HttpOnly", token)))
}

/// 退出登录
#[DELETE("/api/login/logout")]
pub async fn logout() {
    web::API_TOKEN.store(Arc::new(Default::default()));
}

/// 忘记密码
#[GET("/api/login/forget")]
pub async fn forget() -> String {
    let account_path = Path::new(web::ACCOUNT_PATH);
    if fs::exists(account_path).unwrap() {
        format!("请删除文件:{}后重新设置登录信息!", web::ACCOUNT_PATH)
    } else {
        "第一次登录时请直接输入用户名及密码,系统会根据您输入的内容自动创建用户".to_string()
    }
}

/// 登录表单数据结构
#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct LoginForm {
    //管理员登录名
    #[validate(length(min = 1, message = "用户名不能为空"))]
    pub name: String,

    //管理员登录密码
    #[validate(length(min = 1, message = "密码不能为空"))]
    pub pwd: String,
}

/// 登录状态信息结构体
pub struct LoggedInfo {
    //登录状态token
    pub token: String,

    //登录时间戳，单位毫秒
    pub last_time: u64,
}
