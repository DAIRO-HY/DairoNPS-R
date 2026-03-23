use std::time::{SystemTime, UNIX_EPOCH};
use rusqlite::Connection;

// func init() {

// 	//指定删除时间戳
// 	deleteBeforceTime := time.Now().AddDate(-2, 0, 0).Unix()
// 	DBUtil.ExecIgnoreError("delete from date_data_size where date < ?", deleteBeforceTime)
// }

// /**
//  * 添加一条统计
//  * @param 隧道id
//  * @param inData 入网流量
//  * @param outData 出网流量
//  */
    pub fn add(conn: &Connection, channel_id: i64, in_data: i64, out_data: i64) {

        //当前时间戳（秒）
        let date = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
        let sql =
            "insert into channel_data_size(channel_id,date,in_data,out_data)values(?,?,?,?)";
            
        // let mut conn = connection().await;
        // let tx = conn.transaction().unwrap();
        // tx.execute(sql, (channel_id, date, in_data, out_data)).unwrap();
        // tx.commit().unwrap();
        // drop(conn);

        // let mut conn = connection().await;
        // let tx = conn.transaction().unwrap();
        // add123(channel_id, in_data, out_data, &tx);
        // drop(tx);
        // drop(conn);

        conn.execute(sql, (channel_id, date, in_data, out_data)).unwrap();
    }

// /**
//  * 获取数据流量日志列表
//  * @param clientId 客户端id
//  * @param channelId 隧道ID
//  * @param startTime 开始时间
//  * @param endTime 结束时间
//  * @return 数据流量统计列表
//  */
// func SelectList(
// 	clientId int,
// 	channelId int,
// 	forwardId int,
// 	startTime int64,
// 	endTime int64,
// ) []*dto.DateDataSizeDto {
// 	var sql string
// 	if clientId == 0 && channelId == 0 && forwardId == 0 { //所有的统计
// 		sql = "select date,inData,outData from date_data_size where date between ? and ?"
// 		return DBUtil.SelectList[dto.DateDataSizeDto](sql, startTime, endTime)
// 	} else if clientId != 0 { //统计某个客户端
// 		sql = "select date,inData,outData from date_data_size where channelId in (select id from channel where clientId = ?) and date between ? and ?"
// 		return DBUtil.SelectList[dto.DateDataSizeDto](sql, clientId, startTime, endTime)
// 	} else if channelId != 0 { //统计某个隧道
// 		sql = "select date,inData,outData from date_data_size where channelId = ? and date between ? and ?"
// 		return DBUtil.SelectList[dto.DateDataSizeDto](sql, channelId, startTime, endTime)
// 	} else if forwardId != 0 { //统计某个端口转发
// 		sql = "select date,inData,outData from date_data_size where forwardId = ? and date between ? and ?"
// 		return DBUtil.SelectList[dto.DateDataSizeDto](sql, forwardId, startTime, endTime)
// 	} else {
// 		return nil
// 	}
// }

// // 通过隧道ID删除
// func DeleteByChannelId(channelId int) {
// 	sql := "delete from date_data_size where channelId = ?"
// 	DBUtil.ExecIgnoreError(sql, channelId)
// }

// // 通过转发ID删除
// func DeleteByForward(forwardId int) {
// 	sql := "delete from date_data_size where forwardId = ?"
// 	DBUtil.ExecIgnoreError(sql, forwardId)
// }

// // 通过客户端ID删除
// func DeleteByClientId(clientId int) {
// 	sql := "delete from date_data_size where channelId in (select id from channel where clientId = ?)"
// 	DBUtil.ExecIgnoreError(sql, clientId)
// }