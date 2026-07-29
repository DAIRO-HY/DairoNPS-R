-- 通过key获取客户端
-- @name select_by_key
-- @return Client
-- @param key:&str
select * from client where key  = :key;
-- ----------------------------------------------------
--  设置可用状态
-- @name toggle_enable
-- @param id:i64
-- @param is_enabled:bool
update client set is_enabled = :is_enabled where id = :id;
-- ----------------------------------------------------
--  设置客户端连接信息
-- @name set_connection_info
-- @param id:i64
-- @param ip:String
-- @param client_version:String
-- @param last_login_date:i64
update client set ip = :ip, client_version = :client_version, last_login_date = :last_login_date where id = :id;
-- ----------------------------------------------------
--  同步入出网流量
-- @name set_data_len
-- @param id:i64
-- @param in_len:i64
-- @param out_len:i64
update client set in_len = in_len + :in_len,out_len = out_len + :out_len where id = :id;
-- ----------------------------------------------------
        