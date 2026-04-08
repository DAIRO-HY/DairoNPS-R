-- 客户端表
-- drop table client;
CREATE TABLE IF NOT EXISTS client -- 客户端表
(
    id              INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    name            VARCHAR(32)                       NOT NULL,
    key             VARCHAR(32)                       NOT NULL UNIQUE,                                 -- 客户端验证秘钥
    client_version  VARCHAR(10),                                                                       -- 客户端版本
    ip              VARCHAR(128),                                                                      -- 客户端ip地址
    in_data         BIGINT                            NOT NULL DEFAULT 0,                              -- 入网流量
    out_data        BIGINT                            NOT NULL DEFAULT 0,                              -- 出网流量
    online_state    INT8                              NOT NULL DEFAULT 0,                              -- 在线状态
    enable_state    INT8                              NOT NULL DEFAULT 1,                              -- 启用状态 1:开启  0:停止
    last_login_date BIGINT                            NOT NULL DEFAULT (strftime('%s', 'now') * 1000), -- 最后一次连接时间
    created_at      BIGINT                            NOT NULL DEFAULT 0,                              -- 创建时间
    updated_at      BIGINT                            NOT NULL DEFAULT 0,                              -- 更新时间
    remark          TEXT,                                                                              -- 一些备注信息,错误信息等
    version         BIGINT                            NOT NULL DEFAULT 0,                              -- 用作乐观排他
    deleted         INT8                              NOT NULL DEFAULT 0                               -- 删除标记 1:已删除 0:未删除
);

-- 养殖中的耳标编号不允许重复
CREATE UNIQUE INDEX IF NOT EXISTS idx_unique_fid
    ON client (`key`) where deleted = 0;