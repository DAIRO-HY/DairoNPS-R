-- 删除客户端的流量统计
-- @name delete_by_client_id
-- @param client_id:i64
delete from traffic_stats where client_id = :client_id;

-- ------------------------------

-- 删除隧道的流量统计
-- @name delete_by_channel_id
-- @param channel_id:i64
delete from traffic_stats where channel_id = :channel_id;

-- ------------------------------

-- 删除端口转发的流量统计
-- @name delete_by_forward_id
-- @param forward_id:i64
delete from traffic_stats where forward_id = :forward_id;

-- ------------------------------

-- 查询时间段内的入出网流量
-- @name select_io_len
-- @param start_date:i64
-- @param end_date:i64
-- @return List<DateLen>
select date, in_len, out_len from traffic_stats where date between :start_date and :end_date;

-- ------------------------------

-- 查询客户端时间段内的入出网流量
-- @name select_io_len_by_client
-- @param client_id:i64
-- @param start_date:i64
-- @param end_date:i64
-- @return List<DateLen>
select date, in_len, out_len from traffic_stats where client_id = :client_id and date between :start_date and :end_date;

-- ------------------------------

-- 查询隧道时间段内的入出网流量
-- @name select_io_len_by_channel
-- @param channel_id:i64
-- @param start_date:i64
-- @param end_date:i64
-- @return List<DateLen>
select date, in_len, out_len from traffic_stats where channel_id = :channel_id and date between :start_date and :end_date;

-- ------------------------------

-- 查询端口转发时间段内的入出网流量
-- @name select_io_len_by_forward
-- @param forward_id:i64
-- @param start_date:i64
-- @param end_date:i64
-- @return List<DateLen>
select date, in_len, out_len from traffic_stats where forward_id = :forward_id and date between :start_date and :end_date;

-- ------------------------------

-- 删除过期的数据
-- @name delete_expired
-- @param date:i64
delete from traffic_stats where date < :date;
