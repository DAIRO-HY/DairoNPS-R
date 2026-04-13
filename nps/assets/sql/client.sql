-- 客户端表
-- drop table client;
CREATE TABLE IF NOT EXISTS client -- 客户端表
(
    id              INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name            VARCHAR(32)                       NOT NULL,
    key             VARCHAR(32)                       NOT NULL UNIQUE,                                 -- 客户端验证秘钥
    client_version  VARCHAR(10),                                                                       -- 客户端版本
    ip              VARCHAR(128),                                                                      -- 客户端ip地址
    in_len          INTEGER                           NOT NULL DEFAULT 0,                              -- 入网流量
    out_len         INTEGER                           NOT NULL DEFAULT 0,                              -- 出网流量
    is_enabled      BOOLEAN                           NOT NULL DEFAULT 1,                              -- 启用状态 1:开启  0:停止
    last_login_date INTEGER                           NOT NULL DEFAULT (strftime('%s', 'now') * 1000), -- 最后一次连接时间
    created_at      INTEGER                           NOT NULL DEFAULT 0,                              -- 创建时间
    updated_at      INTEGER                           NOT NULL DEFAULT 0,                              -- 更新时间
    remark          TEXT,                                                                              -- 一些备注信息,错误信息等
    version         INTEGER                           NOT NULL DEFAULT 0                               -- 用作乐观排他
);