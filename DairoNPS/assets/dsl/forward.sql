--  通过客户端ID查询所有激活的通道
-- @name select_enabled
-- @return List<Forward>
select * from forward where is_enabled = 1;
-- ----------------------------------------------------
--  设置可用状态
-- @name toggle_enable
-- @param id:i64
-- @param is_enabled:bool
update forward set is_enabled = :is_enabled where id = :id;
-- ----------------------------------------------------
--  同步入出网流量
-- @name set_data_len
-- @param id:i64
-- @param in_len:i64
-- @param out_len:i64
update forward set in_len = :in_len,out_len = :out_len where id = :id;
-- ----------------------------------------------------
--  设置错误消息
-- @name set_error
-- @param id:i64
-- @param error:String
update forward set error = :error where id = :id;
-- ----------------------------------------------------
--  清除错误消息
-- @name clear_error
-- @param id:i64
update forward set error = null where id = :id;
-- ----------------------------------------------------
        