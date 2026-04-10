use std::time::{SystemTime, UNIX_EPOCH};

/// 获取系统时间戳（毫秒）
pub fn current_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
