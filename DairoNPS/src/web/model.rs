
use serde::{Deserialize, Serialize};

/// 登录表单数据结构
#[derive(Deserialize, Serialize, Debug)]
pub struct ResultData<T> {
    pub code: usize,
    pub msg: String,
    pub data: T,
}


#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DropdownData<L, V> {

    /** 标题 **/
    pub label: L,

    /** 值 **/
    pub value: V,
}