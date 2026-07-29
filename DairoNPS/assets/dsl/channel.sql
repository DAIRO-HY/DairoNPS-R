--  通过客户端ID查询所有激活的通道
-- @name select_active_by_client_id
-- @return List<Channel>
-- @param client_id:i64
select channel.* from channel
    left join client on channel.client_id = client.id
    where channel.client_id = :client_id and client.is_enabled = 1 and channel.is_enabled = 1;
-- ----------------------------------------------------
--  设置可用状态
-- @name toggle_enable
-- @param id:i64
-- @param is_enabled:bool
update channel set is_enabled = :is_enabled where id = :id;
-- ----------------------------------------------------
--  同步入出网流量
-- @name set_data_len
-- @param id:i64
-- @param in_len:i64
-- @param out_len:i64
update channel set in_len = :in_len,out_len = :out_len where id = :id;
-- ----------------------------------------------------
-- 设置错误消息
-- @name set_error
-- @param id:i64
-- @param error:String
update channel set error = :error where id = :id;
-- ----------------------------------------------------
-- 清除错误消息
-- @name clear_error
-- @param id:i64
update channel set error = null where id = :id;
-- ----------------------------------------------------
        