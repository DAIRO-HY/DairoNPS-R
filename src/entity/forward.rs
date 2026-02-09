//package dto

// 代理服务器Dto
struct ForwardDto {

	//代理服务ID
	Id: u64,

	//代理服务名
	Name: String,

	//服务端端口
	Port: u64,

	//目标端口(ip:端口)
	TargetPort: String,

	//入网流量
	InData: u64,

	//出网流量
	OutData: u64,

	//启用状态 1:开启  0:停止
	EnableState: u64,

	//黑白名单开启状态 0:关闭 1:白名单 2:黑名单
	AclState: u64,

	//创建时间
	Date: u64,

	//一些备注信息,错误信息等
	Remark: String,

	//错误信息
	Error: String,
}
