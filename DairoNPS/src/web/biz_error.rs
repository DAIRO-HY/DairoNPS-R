use crate::web::model::ResultData;
use hyper::StatusCode;
use hyperoute_core::IntoErrorResponse;
use std::collections::HashMap;

/// 业务错误，handler返回Err时转换成错误响应
/// 前端约定(api-http.js)：
/// - 401：跳转到登录页
/// - JSON且code=2：表单字段错误，data为 字段名 -> 错误消息列表
/// - 其他：直接显示响应文本
#[derive(Debug)]
pub struct BizError {
    status: StatusCode,
    body: String,
    is_json: bool,
}

impl BizError {
    /// 业务错误消息，状态码为500
    pub fn msg(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: msg.into(),
            is_json: false,
        }
    }

    /// 未登录或登录已过期
    pub fn unauthorized() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            body: String::new(),
            is_json: false,
        }
    }

    /// 单个字段错误，状态码为500
    pub fn field(field: &str, msg: &str) -> Self {
        Self::field_map(
            StatusCode::INTERNAL_SERVER_ERROR,
            HashMap::from([(field.to_string(), vec![msg.to_string()])]),
        )
    }

    /// 将表单验证错误转换为字段错误，状态码为500
    pub fn fields(errors: validator::ValidationErrors) -> Self {
        let field_error_map: HashMap<String, Vec<String>> = errors
            .field_errors()
            .into_iter()
            .map(|(field, errs)| {
                (
                    field.to_string(),
                    errs.iter()
                        .filter_map(|err| err.message.as_ref().map(|it| it.to_string()))
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
        Self::field_map(StatusCode::INTERNAL_SERVER_ERROR, field_error_map)
    }

    /// 请求参数错误，状态码为400
    pub fn param(field: &str, msg: &str) -> Self {
        Self::field_map(
            StatusCode::BAD_REQUEST,
            HashMap::from([(field.to_string(), vec![msg.to_string()])]),
        )
    }

    fn field_map(status: StatusCode, data: HashMap<String, Vec<String>>) -> Self {
        let result = ResultData {
            code: 2,
            msg: "表单验证失败".to_string(),
            data,
        };
        Self {
            status,
            body: serde_json::to_string(&result).unwrap(),
            is_json: true,
        }
    }
}

impl IntoErrorResponse for BizError {
    fn status(&self) -> StatusCode {
        self.status
    }

    fn message(&self) -> String {
        self.body.clone()
    }

    fn headers(&self) -> Vec<(&'static str, String)> {
        let content_type = if self.is_json {
            "application/json"
        } else {
            "text/plain; charset=utf-8"
        };
        vec![("content-type", content_type.to_string())]
    }
}
