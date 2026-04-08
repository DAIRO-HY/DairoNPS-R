use thiserror::Error;

#[derive(Error, Debug)]
pub enum NpsError {
    #[error("已经达到最大连接数,拒绝新连接")]
    PoolIsFull,

    #[error("系统级别的错误")]
    IoError(std::io::Error),
}

// impl std::error::Error for NpsError {}

/// 让别的错误自动转成你的错误（非常重要🔥）,这样你就可以直接用 ?：
impl From<std::io::Error> for NpsError {
    fn from(err: std::io::Error) -> Self {
        NpsError::IoError(err)
    }
}