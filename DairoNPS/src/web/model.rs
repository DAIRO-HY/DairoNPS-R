use serde::{Deserialize, Serialize};

/// 登录表单数据结构
#[derive(Deserialize, Serialize, Debug)]
pub struct ResultData<T> {
    pub code: usize,
    pub msg: String,
    pub data: T,
}

/// 从查询参数中提取id
#[derive(Deserialize, Serialize, Debug)]
pub struct IdQuery {
    pub id: i64,
}

/// 从查询参数中获取一个参数
#[derive(Deserialize, Serialize, Debug)]
pub struct SingleQuery<T> {
    pub value: T,
}
