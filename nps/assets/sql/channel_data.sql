-- 隧道流量统计表
CREATE TABLE IF NOT EXISTS channel_data
(
    channel_id INTEGER NOT NULL,           -- 隧道id
    in_data    INTEGER  NOT NULL DEFAULT 0, -- 入网流量
    out_data   INTEGER  NOT NULL DEFAULT 0, -- 出网流量
    date       INTEGER  NOT NULL            -- 统计时间（年月日时分秒）
);
CREATE INDEX idx_channel_data_channel_id ON channel_data (channel_id);