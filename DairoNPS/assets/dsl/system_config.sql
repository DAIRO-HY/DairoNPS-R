-- 获取系统配置
-- @name get
-- @return SystemConfig
select * from system_config;

-- ------------------------------

-- 同步入出网流量
-- @name update_data_io
-- @param in_len:i64
-- @param out_len:i64
update system_config set in_len = in_len + :in_len, out_len = out_len + :out_len;
