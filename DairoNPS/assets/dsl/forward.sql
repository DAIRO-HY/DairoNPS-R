-- 查询所有启用的端口转发
-- @name select_enabled
-- @return List<Forward>
select * from forward where is_enabled = 1;

-- ------------------------------

-- 设置可用状态
-- @name toggle_enable
-- @param id:i64
-- @param is_enabled:bool
update forward set is_enabled = :is_enabled where id = :id;

-- ------------------------------

-- 累加入出网流量(参数为本次增量)
-- @name set_data_len
-- @param id:i64
-- @param in_len:i64
-- @param out_len:i64
update forward set in_len = in_len + :in_len, out_len = out_len + :out_len where id = :id;

-- ------------------------------

-- 设置错误消息
-- @name set_error
-- @param id:i64
-- @param error:String
update forward set error = :error where id = :id;

-- ------------------------------

-- 清除错误消息
-- @name clear_error
-- @param id:i64
update forward set error = null where id = :id;

-- ------------------------------

-- 查询使用指定端口的端口转发数量(排除指定id的端口转发)
-- @name count_by_server_port
-- @param server_port:i64
-- @param exclude_id:i64
-- @return i64
select count(*) from forward where server_port = :server_port and id != :exclude_id;
