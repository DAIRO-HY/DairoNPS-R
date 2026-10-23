-- 通过客户端ID查询所有激活的通道
-- @name select_active_by_client_id
-- @param client_id:i64
-- @return List<Channel>
select channel.* from channel
    left join client on channel.client_id = client.id
    where channel.client_id = :client_id and client.is_enabled = 1 and channel.is_enabled = 1;

-- ------------------------------

-- 设置可用状态
-- @name toggle_enable
-- @param id:i64
-- @param is_enabled:bool
update channel set is_enabled = :is_enabled where id = :id;

-- ------------------------------

-- 累加入出网流量(参数为本次增量)
-- @name set_data_len
-- @param id:i64
-- @param in_len:i64
-- @param out_len:i64
update channel set in_len = in_len + :in_len, out_len = out_len + :out_len where id = :id;

-- ------------------------------

-- 设置错误消息
-- @name set_error
-- @param id:i64
-- @param error:String
update channel set error = :error where id = :id;

-- ------------------------------

-- 清除错误消息
-- @name clear_error
-- @param id:i64
update channel set error = null where id = :id;

-- ------------------------------

-- 删除客户端下的所有隧道
-- @name delete_by_client_id
-- @param client_id:i64
delete from channel where client_id = :client_id;

-- ------------------------------

-- 查询使用指定端口的隧道数量(排除指定id的隧道)
-- @name count_by_server_port
-- @param server_port:i64
-- @param mode:i64
-- @param exclude_id:i64
-- @return i64
select count(*) from channel where server_port = :server_port and mode = :mode and id != :exclude_id;
