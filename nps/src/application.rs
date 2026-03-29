use std::sync::atomic::{AtomicBool, Ordering};
use std::env;
use clap::Parser;

/// 标记是否需要重启
static IS_NEED_RESTART: AtomicBool = AtomicBool::new(false);
pub fn restart_mark() {
    IS_NEED_RESTART.store(true, Ordering::Release);
}
pub fn is_need_restart() -> bool {
    IS_NEED_RESTART.load(Ordering::Acquire)
}

/// 标记是否正在重启
static IS_RESTARTING: AtomicBool = AtomicBool::new(false);
pub fn is_restarting() -> bool {
    IS_RESTARTING.load(Ordering::Acquire)
}

/// 标记是否是axum触发的退出
pub static IS_AXUM_DROP: AtomicBool = AtomicBool::new(false);

/// 标记是否退出了NPS服务端监听
pub static IS_NPS_SERVER_DROP: AtomicBool = AtomicBool::new(false);

/// 重启函数，设置标记并退出程序
pub async fn restart() {
    IS_NEED_RESTART.store(false, Ordering::Release);
    IS_RESTARTING.store(true, Ordering::Release);

    println!("准备关闭Axum服务...");
    loop{
        // 通知axum退出监听
        crate::web::router::SHUTDOWN_NOTIFY.notify_waiters();
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        if IS_AXUM_DROP.load(Ordering::Acquire) {
            break;
        }
    }
    println!("Axum服务已关闭");

    println!("准备关闭NPS服务端监听...");
    loop{
        // 通知NPS服务端退出监听
        crate::nps::nps_client::tcp_client::tcp_client_accept::SHUTDOWN_NOTIFY.notify_waiters();
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        if IS_NPS_SERVER_DROP.load(Ordering::Acquire) {
            break;
        }
    }
    println!("NPS服务端监听已关闭");

    println!("准备重启...");
    let exe = env::current_exe().expect("获取程序路径失败");
    let mut args: Vec<String> = env::args().skip(1).collect();
    let arg = Args::try_parse().unwrap();
    if !arg.is_restart_mode {//防止重复添加重启参数
        args.push("--is-restart-mode".to_string());
    }
    println!("exe: {:?}, args: {:?}", exe, args);
    std::process::Command::new(exe).args(args).spawn().expect("重启失败");
    std::process::exit(0);
}

#[derive(Parser, Debug)]
#[command(name = "myapp", version, about = "示例程序")]
pub struct Args {

    /// 是否重启模式
    #[arg(short, long)]
    pub is_restart_mode: bool,
}