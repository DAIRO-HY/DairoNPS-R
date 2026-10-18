-- 流量统计表
CREATE TABLE IF NOT EXISTS traffic_stats
(
    client_id  INTEGER NOT NULL,           -- 客户端id
    channel_id INTEGER NOT NULL,           -- 隧道id
    forward_id INTEGER NOT NULL,           -- 端口转发id
    in_len    INTEGER NOT NULL DEFAULT 0, -- 入网流量
    out_len   INTEGER NOT NULL DEFAULT 0, -- 出网流量
    date       INTEGER NOT NULL            -- 统计时间（年月日时分秒）
);
CREATE INDEX IF NOT EXISTS traffic_stats_idx_client_id ON traffic_stats (client_id);
CREATE INDEX IF NOT EXISTS traffic_stats_idx_channel_id ON traffic_stats (channel_id);
CREATE INDEX IF NOT EXISTS traffic_stats_idx_forward_id ON traffic_stats (forward_id);