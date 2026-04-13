use thiserror::Error;

#[derive(Error, Debug)]
pub enum NpsError {
    #[error("已经达到最大连接数,拒绝新连接")]
    PoolIsFull,

    #[error("系统级别的错误")]
    IoError(std::io::Error),

    #[error("sqlx执行的错误")]
    SqlxError(sqlx::Error),

    #[error("未知的Flag")]
    UnknowFlagError,

    #[error("发送数据失败")]
    SendDataError,

    #[error("其他错误")]
    OtherError(String),
}

// impl std::error::Error for NpsError {}

/// 让别的错误自动转成你的错误（非常重要🔥）,这样你就可以直接用 ?：
impl From<std::io::Error> for NpsError {
    fn from(err: std::io::Error) -> Self {
        NpsError::IoError(err)
    }
}
impl From<sqlx::Error> for NpsError {
    fn from(err: sqlx::Error) -> Self {
        NpsError::SqlxError(err)
    }
}