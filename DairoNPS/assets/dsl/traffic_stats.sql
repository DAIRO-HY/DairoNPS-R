-- @name delete_by_client_id
-- @param client_id:i64
delete from traffic_stats where client_id = :client_id;;
-- ----------------------------------------------------
-- @name delete_by_channel_id
-- @param channel_id:i64
delete from traffic_stats where channel_id = :channel_id;;
-- ----------------------------------------------------
-- @name delete_by_forward_id
-- @param forward_id:i64
delete from traffic_stats where forward_id = :forward_id;;
-- ----------------------------------------------------
-- @name select_io_len
-- @return List<DateLen>
-- @param start_date:i64
-- @param end_date:i64
select date,in_len,out_len from traffic_stats where date between :start_date and :end_date;
-- ----------------------------------------------------
-- @name select_io_len_by_client
-- @return List<DateLen>
-- @param client_id:i64
-- @param start_date:i64
-- @param end_date:i64
select date,in_len,out_len from traffic_stats where client_id = :client_id and date between :start_date and :end_date;
-- ----------------------------------------------------
-- @name select_io_len_by_channel
-- @return List<DateLen>
-- @param channel_id:i64
-- @param start_date:i64
-- @param end_date:i64
select date,in_len,out_len from traffic_stats where channel_id = :channel_id and date between :start_date and :end_date;
-- ----------------------------------------------------
-- @name select_io_len_by_forward
-- @return List<DateLen>
-- @param forward_id:i64
-- @param start_date:i64
-- @param end_date:i64
select date,in_len,out_len from traffic_stats where forward_id = :forward_id and date between :start_date and :end_date;
-- ----------------------------------------------------
--  删除过期的数据
-- @name delete_expired
-- @param date:i64
delete from traffic_stats where date < :date;;
-- ----------------------------------------------------
        