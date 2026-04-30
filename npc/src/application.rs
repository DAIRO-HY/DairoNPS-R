use arc_swap::ArcSwap;
use clap::Parser;
use np_common::time_util;
use std::sync::Arc;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, AtomicUsize};
use tokio::sync::{Mutex, Notify, RwLock};

/// 程序版本
pub const VERSION: &str = "1.0.0";

/// 标记是否已经关闭
// pub static IS_CLOSED: AtomicBool = AtomicBool::new(false);

/// 用来接收关闭通知的全局异步通知器
pub static APP_CLOSER: Notify = Notify::const_new();

/// 标记是否正在运行,防止重复启动
pub static IS_RUNNING: Mutex<bool> = Mutex::const_new(false);

/// 标记NPC监听是否正在运行
pub static IS_NPC_RUNNING: Mutex<bool> = Mutex::const_new(false);

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

/// 用来接收关闭通知的全局异步通知器
pub static ARGS: LazyLock<Argument> = LazyLock::new(|| Argument::try_parse().unwrap());

/// 桥接数量
pub static BRIDGE_COUNT: AtomicUsize = AtomicUsize::new(0);

/// 连接池数量
pub static POOL_COUNT: AtomicUsize = AtomicUsize::new(0);

/// 客户端ID
pub static CLIENT_ID: AtomicI64 = AtomicI64::new(0);

/// 客户端端加密秘钥
pub static SECURITY_KEY: RwLock<[u8; 256]> = RwLock::const_new([0u8; 256]);

// /// 重启函数，设置标记并退出程序
// pub async fn restart() {
//     IS_NEED_RESTART.store(false, Ordering::Release);
//     IS_RESTARTING.store(true, Ordering::Release);
//
//     println!("准备关闭服务...");
//     loop {
//         // 通知退出监听
//         println!("正在关闭服务...");
//         SHUTDOWN_NOTIFY.notify_waiters();
//         tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
//         if !nps::CHANNEL_LIVE_MAP.lock().await.is_empty() {
//             //等待所有隧道代理监听停止,否则可能导致下次监听同一端口失败
//             continue;
//         }
//         if !IS_AXUM_DROP.load(Ordering::Acquire) {
//             //等待axum触发的退出完成,否则可能导致下次监听同一端口失败
//             continue;
//         }
//         if !IS_NPS_SERVER_DROP.load(Ordering::Acquire) {
//             //等待NPS服务端监听退出完成,否则可能导致下次监听同一端口失败
//             continue;
//         }
//         break;
//     }
//     println!("NPS服务端监听已关闭");
//
//     println!("准备重启...");
//     let exe = env::current_exe().expect("获取程序路径失败");
//     let mut args: Vec<String> = env::args().skip(1).collect();
//     let arg = Args::try_parse().unwrap();
//     if !arg.is_restart_mode {
//         //防止重复添加重启参数
//         args.push("--is-restart-mode".to_string());
//     }
//     println!("exe: {:?}, args: {:?}", exe, args);
//     std::process::Command::new(exe)
//         .args(args)
//         .spawn()
//         .expect("重启失败");
//     std::process::exit(0);
// }
//
// #[derive(Parser, Debug)]
// #[command(name = "myapp", version, about = "示例程序")]
// pub struct Args {
//     /// 是否重启模式
//     #[arg(short, long)]
//     pub is_restart_mode: bool,
// }

/// 程序启动参数
///
#[derive(Parser, Debug)]
#[command(name = "npc", version, about = "示例程序")]
pub struct Argument {
    // // 服务器
    // var Host string
    //
    // // 服务器端TCP端口
    // var TcpPort string
    //
    // // 服务器端UDP端口
    // var UdpPort string
    //
    // // 认证秘钥
    // var Key string
    //
    // // 客户端id,该值有服务器端返回
    // var ClientId int
    #[arg(long, default_value = "127.0.0.1")]
    pub host: String,

    #[arg(short, long, default_value = "1781")]
    pub tcp_port: u16,

    #[arg(short, long, default_value = "1782")]
    pub udp_port: u16,

    #[arg(short, long, default_value = "njeHds*fs4tfsd")]
    pub key: String,
}
