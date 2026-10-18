use serde::{Deserialize, Serialize};
use validator::Validate;

// 表单结构体放在controller之外：hyperoute会为同文件内的请求体结构体生成multipart解析，
// 而multipart暂不支持Option字段，前端只提交urlencoded，所以这里只走serde解析

// 客户端编辑信息
#[derive(Deserialize, Debug, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ClientEdit {
    // id
    pub id: i64,

    // 名称
    #[validate(length(min = 1, max = 32, message = "名称不能为空;长度不能超过32位"))]
    pub name: String,

    // 乐观排他用的版本号
    pub version: i64,

    // 连接认证秘钥
    #[validate(length(min = 1, max = 32, message = "认证秘钥不能为空;长度不能超过32位"))]
    pub key: String,

    // 一些备注信息,错误信息等
    pub remark: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ChannelEdit {
    pub id: i64,
    pub client_id: i64,

    #[validate(length(min = 1, max = 32, message = "名称不能为空;长度不能超过32位"))]
    pub name: String,
    pub mode: i64,
    pub server_port: i64,
    pub target_port: String,
    pub security_state: i64,

    ///是否统计流量
    pub is_stats_traffic: bool,
    pub remark: Option<String>,

    // 乐观排他用的版本号
    pub version: i64,
}

#[derive(Debug, Default, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ForwardEdit {
    pub id: i64,

    #[validate(length(min = 1, max = 32, message = "名称不能为空;长度不能超过32位"))]
    pub name: String,
    pub server_port: i64,
    pub target_port: String,

    ///是否统计流量
    pub is_stats_traffic: bool,
    pub remark: Option<String>,
}
