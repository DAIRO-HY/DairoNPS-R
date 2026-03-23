use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// 流量计数（入/出），底层使用 `Arc<AtomicU64>` 以便跨任务共享并原子更新
#[derive(Clone, Debug)]
pub struct DataTotal {
    pub in_bytes: Arc<AtomicU64>,
    pub out_bytes: Arc<AtomicU64>,
}

/// 小 trait：把各种整数转换为 u64
pub trait ToU64 {
    fn to_u64(self) -> u64;
}

macro_rules! impl_to_u64 {
    ($($t:ty),+) => {
        $(impl ToU64 for $t {
            fn to_u64(self) -> u64 {
                self as u64
            }
        })+
    };
}

// 为常见无符号类型和 usize 实现
impl_to_u64!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize);


impl DataTotal {

    // 创建新的流量计数实例，初始值为0
    pub fn new() -> Self {
        Self {
            in_bytes: Arc::new(AtomicU64::new(0)),
            out_bytes: Arc::new(AtomicU64::new(0)),
        }
    }

    // 创建新的流量计数实例，初始值为指定的入站和出站字节数
    pub fn from<T: ToU64>(in_bytes: T, out_bytes: T) -> Self {
        Self {
            in_bytes: Arc::new(AtomicU64::new(in_bytes.to_u64())),
            out_bytes: Arc::new(AtomicU64::new(out_bytes.to_u64())),
        }
    }

    // 原子地增加入站流量
    pub fn add_in<T: ToU64>(&self, v: T) {
        self.in_bytes.fetch_add(v.to_u64(), Ordering::Relaxed);
    }

    // 获取当前入站流量总和
    pub fn add_out<T: ToU64>(&self, v: T) {
        self.out_bytes.fetch_add(v.to_u64(), Ordering::Relaxed);
    }

    // 获取当前入站流量总和
    pub fn load_in(&self) -> u64 {
        self.in_bytes.load(Ordering::Relaxed)
    }

    // 获取当前出站流量总和
    pub fn load_out(&self) -> u64 {
        self.out_bytes.load(Ordering::Relaxed)
    }
}


// 实现 PartialEq 以便比较两个 DataTotal 实例是否相等（入站和出站流量都相等）
impl PartialEq for DataTotal {
    fn eq(&self, other: &Self) -> bool {
        self.load_in() == other.load_in() && self.load_out() == other.load_out()
    }
}
