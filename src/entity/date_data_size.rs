//package dto

// 流量统计表DTO
struct DateDataSizeDto {

	//隧道id
	ChannelId: u64,

	//年月日时分秒
	Date: u64,

	//入网流量
	InData: u64,

	//出网流量
	OutData: u64,
}
