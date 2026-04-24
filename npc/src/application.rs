use clap::Parser;
use std::env;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::Notify;
use std::sync::LazyLock;

/// 最后一次收到心跳反馈时间
pub static LAST_HEART_TIME: AtomicU64 = AtomicU64::new(np_common::time_util::current_millis());

/// 用来生成当前桥接唯一标识
pub static BRIDGE_NEXT_TAG: AtomicU64 = AtomicU64::new(0);

/// 用来接收关闭通知的全局异步通知器
pub static SHUTDOWN_NOTIFY: LazyLock<Arc<Notify>> = LazyLock::new(|| Arc::new(Notify::const_new()));

/// 标记是否需要重启
pub static IS_NEED_RESTART: AtomicBool = AtomicBool::new(false);

/// 标记是否正在重启
pub static IS_RESTARTING: AtomicBool = AtomicBool::new(false);

/// 标记是否是axum触发的退出
pub static IS_AXUM_DROP: AtomicBool = AtomicBool::new(false);

/// 标记是否退出了NPS服务端监听
pub static IS_NPS_SERVER_DROP: AtomicBool = AtomicBool::new(false);

/// 心跳间隔时间
pub const HEART_TIME: u64 = 3000;

/// 数据流量收集统计间隔，单位毫秒
pub const DATA_COLLECT_INTERVAL: u64 = 1000;

/// 数据流量收集插入数据库间隔，单位毫秒
pub const DATA_COLLECT_INSERT_INTERVAL: u64 = 6000;


/// 用来接收关闭通知的全局异步通知器
pub static ARGS: LazyLock<Argument> = LazyLock::new(|| Argument::try_parse().unwrap());

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
#[command(name = "myapp", version, about = "示例程序")]
pub struct Argument{


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

    #[arg(short, long, default_value = "127.0.0.1")]
    pub host: String,

    #[arg(short, long, default_value = "1781")]
    pub tcp_port: u16,

    #[arg(short, long, default_value = "1782")]
    pub udp_port: u16,

    #[arg(short, long, default_value = "njeHds*fs4tfsd")]
    pub key: String,
}