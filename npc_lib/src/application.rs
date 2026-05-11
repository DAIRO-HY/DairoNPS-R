use std::string::ToString;
use clap::Parser;
use np_common::time_util;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, AtomicUsize};
use std::sync::LazyLock;
use tokio::sync::{Mutex, Notify, RwLock};

/// 程序版本
pub const VERSION: &str = "1.0.0";

/// 标记是否已经关闭
// pub static IS_CLOSED: AtomicBool = AtomicBool::new(false);

/// 用来接收关闭通知的全局异步通知器
pub static APP_CLOSER: Notify = Notify::const_new();

/// 标记是否正在运行,防止重复启动
pub static IS_OPENED: Mutex<bool> = Mutex::const_new(false);

/// 标记NPC监听是否正在运行
pub static IS_NPC_RUNNING: Mutex<bool> = Mutex::const_new(false);

/// NPC服务连接状态
pub static NPC_CLOSER: Notify = Notify::const_new();

/// 最后一次收到心跳反馈时间
pub static LAST_HEART_TIME: LazyLock<AtomicU64> =
    LazyLock::new(|| AtomicU64::new(time_util::current_millis() - CHECK_HEART_TIME - 1));

/// 标记是否退出了NPS服务端监听
pub static IS_NPS_SERVER_DROP: AtomicBool = AtomicBool::new(false);

/// 心跳间隔时间
pub const HEART_TIME: u64 = 3000;

/// 每隔一段时间检测心跳存活状态
pub const CHECK_HEART_TIME: u64 = HEART_TIME * 3;

/// 桥接数量
pub static BRIDGE_COUNT: AtomicUsize = AtomicUsize::new(0);

/// 连接池数量
pub static POOL_COUNT: AtomicUsize = AtomicUsize::new(0);

/// 客户端ID
pub static CLIENT_ID: AtomicI64 = AtomicI64::new(0);

/// 客户端端加密秘钥
pub static SECURITY_KEY: RwLock<[u8; 256]> = RwLock::const_new([0u8; 256]);

/// NPC运行状态信息
pub static NPC_CONNECT_MSG: LazyLock<RwLock<String>> = LazyLock::new(|| RwLock::new("NPC服务未启动".to_string()));

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
