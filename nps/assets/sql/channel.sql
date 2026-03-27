-- 隧道表
CREATE TABLE IF NOT EXISTS channel
(
    id            INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    client_id      INTEGER     NOT NULL,                                        -- 客户端id
    name          VARCHAR(32) NOT NULL,
    mode          INT8     NOT NULL,                                        -- 隧道模式, 1:TCP  2:UDP
    server_port    INT16     NOT NULL UNIQUE,                                 -- 服务器端端口
    target_port    VARCHAR(32) NOT NULL,                                        -- 目标端口(ip:端口)
    in_data        BIGINT      NOT NULL DEFAULT 0,                              -- 入网流量
    out_data       BIGINT      NOT NULL DEFAULT 0,                              -- 出网流量
    enable_state   INT8     NOT NULL DEFAULT 1,                              -- 启用状态 1:开启  0:停止
    security_state INT8     NOT NULL DEFAULT 0,                              -- 是否加密传输
    acl_state      INT8     NOT NULL DEFAULT 0,                              -- 黑白名单开启状态 0:关闭 1:白名单 2:黑名单
    created_at    BIGINT      NOT NULL DEFAULT 0,                              -- 创建时间
    updated_at    BIGINT      NOT NULL DEFAULT 0,                              -- 更新时间
    remark        TEXT,                                                        -- 一些备注信息,错误信息等
    error         TEXT,                                                         -- 错误信息
    version       BIGINT      NOT NULL DEFAULT 0,                             -- 用作乐观排他
    deleted       INT8        NOT NULL DEFAULT 0                               -- 删除标记 1:已删除 0:未删除
);