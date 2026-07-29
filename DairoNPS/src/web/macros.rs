use axum::http::Response;

// 用法：
//   biz_error!("message");
// 展开为：
//   (StatusCode::INTERNAL_SERVER_ERROR, "message")
#[macro_export]
macro_rules! biz_error {
    // 形式一：仅消息字面量
    ($msg:literal) => {
        Response::error($msg)
        // (axum::http::StatusCode::INTERNAL_SERVER_ERROR, $msg).into_response()
    };

    // 形式二：String和&str都支持,如果是String零拷贝，如果是&str则转换为String
    ($msg:expr) => {
        Response::error_str($msg)
        // (axum::http::StatusCode::INTERNAL_SERVER_ERROR, std::convert::Into::<::std::string::String>::into($msg)).into_response()
    };
}

#[macro_export]
macro_rules! biz_errorf {
    // 可变参数格式化
    ($($arg:tt)*) => {
        Response::error_str(std::format!($($arg)*))
        // Err(AppError::InternalError(std::format!($($arg)*)))
        // (axum::http::StatusCode::INTERNAL_SERVER_ERROR, std::format!($($arg)*)).into_response()
    };
}