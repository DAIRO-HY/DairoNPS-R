-- 流量统计表
CREATE TABLE IF NOT EXISTS data_io
(
    client_id  INTEGER NOT NULL,           -- 客户端id
    channel_id INTEGER NOT NULL,           -- 隧道id
    forward_id INTEGER NOT NULL,           -- 端口转发id
    in_len    INTEGER NOT NULL DEFAULT 0, -- 入网流量
    out_len   INTEGER NOT NULL DEFAULT 0, -- 出网流量
    date       INTEGER NOT NULL            -- 统计时间（年月日时分秒）
);
CREATE INDEX IF NOT EXISTS data_io_idx_client_id ON data_io (client_id);
CREATE INDEX IF NOT EXISTS data_io_idx_channel_id ON data_io (channel_id);
CREATE INDEX IF NOT EXISTS data_io_idx_forward_id ON data_io (forward_id);