//package dto

struct ChannelSearchDto {
	Id: u64,

	//客户端id
	ClientId: u64,

	//隧道名
	Name: String,

	//隧道模式, 1:TCP  2:UDP
	Mode: u64,

	//服务端端口
	ServerPort: u64,

	//目标端口(ip:端口)
	TargetPort: String,

	//入网流量
	InData: u64,

	//出网流量
	OutData: u64,

	//启用状态 1:开启  0:停止
	EnableState: u64,

	//是否加密传输
	SecurityState: u64,

	//创建时间
	CreateDate: u64,

	//最后一次更新时间
	UpdateDate: u64,

	//客户端名
	ClientName: String,
}
