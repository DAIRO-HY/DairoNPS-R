use clap::Parser;
use lib_np_common::data_io_len::AtomicDataIOLen;
use std::string::ToString;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI64, AtomicU16, AtomicU64, AtomicUsize};
use tokio::sync::{Notify, RwLock};

/// 程序版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 用来接收关闭通知的全局异步通知器
pub static APP_CLOSER: Notify = Notify::const_new();

/// 标记是否正在运行,防止重复启动
pub static IS_OPENED: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

/// 标记NPC监听是否正在运行
pub static IS_NPC_RUNNING: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

/// NPC服务连接状态
pub static NPC_CLOSER: Notify = Notify::const_new();

/// 最后一次收到心跳反馈时间
pub static LAST_HEART_TIME: AtomicU64 = AtomicU64::new(0);

/// 心跳间隔时间
pub const HEART_TIME: u64 = 3000;

/// 每隔一段时间检测心跳存活状态
pub const CHECK_HEART_TIME: u64 = HEART_TIME * 3;

/// 桥接数量
pub static BRIDGE_COUNT: AtomicU16 = AtomicU16::new(0);

/// 连接池数量
pub static POOL_COUNT: AtomicU16 = AtomicU16::new(0);

/// 客户端ID
pub static CLIENT_ID: AtomicI64 = AtomicI64::new(0);

/// 客户端端加密秘钥
pub static SECURITY_KEY: RwLock<[u8; 256]> = RwLock::const_new([0u8; 256]);

/// NPC运行状态信息
pub static NPC_CONNECT_MSG: LazyLock<Mutex<String>> =
    LazyLock::new(|| Mutex::new("NPC服务未启动".to_string()));

pub static DATA_IO: LazyLock<AtomicDataIOLen> = LazyLock::new(|| AtomicDataIOLen::new());

/// 入网总流量
pub static IN_LEN: AtomicU64 = AtomicU64::new(0);

/// 出网总流量
pub static OUT_LEN: AtomicU64 = AtomicU64::new(0);

/// 程序启动参数
#[derive(Parser, Clone, Debug)]
#[command(name = "npc", version, about = "示例程序")]
pub struct Argument {
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    #[arg(short, long, default_value = "1881")]
    pub tcp_port: u16,

    #[arg(short, long, default_value = "1882")]
    pub udp_port: u16,

    #[arg(short, long, default_value = "njeHds*fs4tfsd")]
    pub key: String,
}
