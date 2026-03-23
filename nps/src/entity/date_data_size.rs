//package dto

// 流量统计表DTO
struct DateDataSizeDto {

	//隧道id
	channel_id: u64,

	//年月日时分秒
	date: u64,

	//入网流量
	in_data: u64,

	//出网流量
	out_data: u64,
}
