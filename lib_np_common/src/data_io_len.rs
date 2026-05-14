use std::ops::Add;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

/// 流量计数（入/出），底层使用 `Arc<AtomicU64>` 以便跨任务共享并原子更新
#[derive(Clone, Debug)]
pub struct AtomicDataIOLen {
    pub in_len: Arc<AtomicU64>,
    pub out_len: Arc<AtomicU64>,
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

impl AtomicDataIOLen {
    // 创建新的流量计数实例，初始值为0
    pub fn new() -> Self {
        Self {
            in_len: Arc::new(AtomicU64::new(0)),
            out_len: Arc::new(AtomicU64::new(0)),
        }
    }

    // 创建新的流量计数实例，初始值为指定的入站和出站字节数
    pub fn from<T: ToU64>(in_len: T, out_len: T) -> Self {
        Self {
            in_len: Arc::new(AtomicU64::new(in_len.to_u64())),
            out_len: Arc::new(AtomicU64::new(out_len.to_u64())),
        }
    }

    // 原子地增加入站流量
    pub fn add_in<T: ToU64>(&self, v: T) {
        self.in_len.fetch_add(v.to_u64(), Ordering::Relaxed);
    }

    // 获取当前入站流量总和
    pub fn add_out<T: ToU64>(&self, v: T) {
        self.out_len.fetch_add(v.to_u64(), Ordering::Relaxed);
    }

    /// 获取当前入网流量
    pub fn load_in(&self) -> u64 {
        self.in_len.load(Ordering::Relaxed)
    }

    /// 获取当前出网流量
    pub fn load_out(&self) -> u64 {
        self.out_len.load(Ordering::Relaxed)
    }

    // 获取当前出站流量总和
    pub fn load(&self) -> DataIOLen {
        DataIOLen {
            in_len: self.in_len.load(Ordering::Relaxed),
            out_len: self.out_len.load(Ordering::Relaxed),
        }
    }
}

// 实现 PartialEq 以便比较两个 BytesIO 实例是否相等（入站和出站流量都相等）
impl PartialEq for AtomicDataIOLen {
    fn eq(&self, other: &Self) -> bool {
        self.load() == other.load()
    }
}

/// 流量计数（入/出）
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DataIOLen {
    pub in_len: u64,
    pub out_len: u64,
}
impl DataIOLen {
    pub fn from<T: ToU64>(in_len: T, out_len: T)->Self{
        Self{
            in_len:in_len.to_u64(),
            out_len:out_len.to_u64(),
        }
    }
}

/// 实现相加
impl Add for DataIOLen {
    type Output = DataIOLen;
    fn add(self, other: DataIOLen) -> DataIOLen {
        DataIOLen {
            in_len: self.in_len + other.in_len,
            out_len: self.out_len + other.out_len,
        }
    }
}