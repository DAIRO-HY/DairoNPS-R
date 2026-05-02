-- 端口转发
CREATE TABLE IF NOT EXISTS forward
(
    id               INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name             VARCHAR(32)                       NOT NULL UNIQUE,    -- 转发名
    server_port      INTEGER                           NOT NULL UNIQUE,    -- 服务器端端口
    target_port      VARCHAR(32)                       NOT NULL,           -- 目标端口(ip:端口)
    acl_state        INTEGER                           NOT NULL DEFAULT 0, -- 黑白名单开启状态 0:关闭 1:白名单 2:黑名单
    in_len           INTEGER                           NOT NULL DEFAULT 0, -- 入网流量
    out_len          INTEGER                           NOT NULL DEFAULT 0, -- 出网流量
    is_enabled       BOOLEAN                           NOT NULL DEFAULT 1, -- 启用状态 1:开启  0:停止
    is_stats_traffic BOOLEAN                           NOT NULL DEFAULT 1, -- 是否统计流量 1:统计  0:不统计
    created_at       INTEGER                           NOT NULL,           -- 创建时间
    remark           TEXT,                                                 -- 一些备注信息,错误信息等
    error            TEXT                                                  -- 错误信息
);
