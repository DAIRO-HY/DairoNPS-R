use thiserror::Error;

#[derive(Error, Debug)]
pub enum NpcError {
    #[error("已经达到最大连接数,拒绝新连接")]
    PoolIsFull,

    #[error("系统级别的错误")]
    IoError(std::io::Error),

    #[error("未知的Flag")]
    UnknowFlagError(u8),

    #[error("无效的Header")]
    InvalidHeader(String),
}

/// 让别的错误自动转成你的错误（非常重要🔥）,这样你就可以直接用 ?：
impl From<std::io::Error> for NpcError {
    fn from(err: std::io::Error) -> Self {
        NpcError::IoError(err)
    }
}