pub struct Channel {

    //隧道ID
    pub id:u64,

    //客户端id
    pub client_id: u64,

    //隧道名
    pub name: String,

    //隧道模式, 1:TCP  2:UDP
    pub mode: u64,

    //服务端端口
    pub server_port: u64,

    //目标端口(ip:端口)
    pub target_port: String,

    //入网流量
    pub in_data: u64,

    //出网流量
    pub out_data: u64,

    //启用状态 1:开启  0:停止
    pub enable_state: u64,

    //是否加密传输
    pub security_state: u64,

    //黑白名单开启状态 0:关闭 1:白名单 2:黑名单
    pub acl_state: u64,

    //创建时间
    pub date: u64,

    //一些备注信息,错误信息等
    pub remark: string,

    //错误信息
    pub error: string,
}
