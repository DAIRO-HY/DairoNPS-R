//package dto

// ClientDto 客户端信息
pub struct Client {

    //id
    pub id: u64,

    //名称
    pub name: string,

    //版本号
    pub version: string,

    //连接认证秘钥
    pub key: string,

    //ip地址
    pub ip: string,

    //入网流量
    pub in_data: u64,

    //出网流量
    pub out_data: u64,

    //在线状态,0:离线 1:在线
    pub online_state: u64,

    //启用状态
    pub enable_state: u64,

    //最后一次连接时间
    pub last_login_date: u64,

    //创建时间
    pub date: u64,

    //一些备注信息,错误信息等
    pub remark: string,
}
