-- 系统配置
CREATE TABLE IF NOT EXISTS  system_config
(
    subsidyEnable   INTEGER NOT NULL, -- 启动补贴申请功能
    subsidyCloseMsg VARCHAR(3000) NOT NULL -- 补贴功能关闭时提示消息
);
