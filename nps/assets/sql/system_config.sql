-- 系统配置
CREATE TABLE IF NOT EXISTS system_config
(
    in_data  INTEGER NOT NULL DEFAULT 0, -- 入网流量
    out_data INTEGER NOT NULL DEFAULT 0  -- 出网流量
);