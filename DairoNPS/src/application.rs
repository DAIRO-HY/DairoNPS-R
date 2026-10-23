use crate::{forward, nps};
use clap::Parser;
use std::env;
use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::Notify;

/// 心跳间隔时间
pub const HEART_TIME: u64 = 3000;

/// 用来生成当前桥接唯一标识
pub static BRIDGE_NEXT_TAG: AtomicU64 = AtomicU64::new(0);

/// 用来接收关闭通知的全局异步通知器
pub static SHUTDOWN_NOTIFY: Notify = Notify::const_new();

/// 标记是否需要重启
pub static IS_NEED_RESTART: AtomicBool = AtomicBool::new(false);

/// 标记是否正在重启
pub static IS_RESTARTING: AtomicBool = AtomicBool::new(false);

/// 标记Web服务监听是否已经退出
pub static IS_WEB_DROP: AtomicBool = AtomicBool::new(false);

/// 标记是否退出了NPS服务端监听
pub static IS_NPS_SERVER_DROP: AtomicBool = AtomicBool::new(false);

/// 用来接收关闭通知的全局异步通知器
pub static ARGS: LazyLock<Argument> = LazyLock::new(|| Argument::try_parse().unwrap());

/// 开发时使用和编译期相同的数据目录(由build.rs注入)
#[cfg(debug_assertions)]
const DEFAULT_DATA_DIR: &str = env!("DAIRO_NPS_DEV_DATA_DIR");

/// 发布时使用程序启动目录下的data目录
#[cfg(not(debug_assertions))]
const DEFAULT_DATA_DIR: &str = "./data";

/// 账号信息文件路径
pub static ACCOUNT_PATH: LazyLock<PathBuf> = LazyLock::new(|| PathBuf::from(&ARGS.data_dir).join("account.txt"));

/// 重启函数，设置标记并退出程序
pub async fn restart() {
    IS_NEED_RESTART.store(false, Ordering::Relaxed);
    IS_RESTARTING.store(true, Ordering::Relaxed);

    println!("准备关闭服务...");
    loop {
        // 通知退出监听
        println!("正在关闭服务...");
        SHUTDOWN_NOTIFY.notify_waiters();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        if !nps::CHANNEL_LIVE_MAP.lock().await.is_empty() {
            //等待所有隧道代理监听停止,否则可能导致下次监听同一端口失败
            continue;
        }
        if !forward::FORWARD_LIVE_MAP.lock().await.is_empty() {
            //等待所有端口转发监听停止,否则可能导致下次监听同一端口失败
            continue;
        }
        if !IS_WEB_DROP.load(Ordering::Acquire) {
            //等待Web服务监听退出完成,否则可能导致下次监听同一端口失败
            continue;
        }
        if !IS_NPS_SERVER_DROP.load(Ordering::Acquire) {
            //等待NPS服务端监听退出完成,否则可能导致下次监听同一端口失败
            continue;
        }
        break;
    }
    println!("NPS服务端监听已关闭");

    println!("准备重启...");
    let exe = env::current_exe().expect("获取程序路径失败");
    let mut args: Vec<String> = env::args().skip(1).collect();
    let arg = Argument::try_parse().unwrap();
    if !arg.is_restart_mode {
        //防止重复添加重启参数
        args.push("--is-restart-mode".to_string());
    }
    println!("exe: {:?}, args: {:?}", exe, args);
    std::process::Command::new(exe)
        .args(args)
        .spawn()
        .expect("重启失败");
    std::process::exit(0);
}

#[derive(Parser, Debug)]
#[command(name = "DairoNPS", version, about = "DairoNPS")]
pub struct Argument {
    /// WEB管理端口
    #[arg(long, env = "DAIRO_NPS_WEB_PORT", default_value = "1880")]
    pub web_port: u16,

    /// 服务端监听TCP端口,客户端通过此端口进行连接
    #[arg(long, env = "DAIRO_NPS_TCP_PORT", default_value = "1881")]
    pub tcp_port: u16,

    /// 服务端监听UDP端口,客户端通过此端口进行连接
    #[arg(long, env = "DAIRO_NPS_UDP_PORT", default_value = "1882")]
    pub udp_port: u16,

    /// 是否重启模式
    #[arg(long)]
    pub is_restart_mode: bool,

    /// 流量数据有效期时间(秒),0:永久有效
    #[arg(long, env = "DAIRO_NPS_TRAFFIC_STATS_EXPIRED", default_value = "0")]
    pub traffic_stats_expired: u64,

    /// 连接池最大数量
    #[arg(long, env = "DAIRO_NPS_MAX_POOL_COUNT", default_value = "6")]
    pub max_pool_count: usize,

    /// 连接池最低数量
    /// 连接池中的Socket在一段时间内无任何操作
    #[arg(long, env = "DAIRO_NPS_MIN_POOL_COUNT", default_value = "1")]
    pub min_pool_count: usize,

    /// 连接池不足时,一次性创建连接数
    #[arg(long, env = "DAIRO_NPS_ADD_POOL_COUNT", default_value = "1")]
    pub add_pool_count: u8,

    /// 读取数据缓存大小
    #[arg(long, env = "DAIRO_NPS_READ_CACHE_SIZE", default_value = "8192")]
    pub read_cache_size: usize,

    /// 每隔一段时间回收长时间不用的连接池（毫秒）
    #[arg(long, env = "DAIRO_NPS_RECYCLE_POOL_TIME_OUT", default_value = "180000")]
    pub recycle_pool_time_out: u64,

    /// 数据流量收集统计间隔，单位毫秒
    #[arg(long, env = "DAIRO_NPS_DATA_COLLECT_INTERVAL", default_value = "1000")]
    pub data_collect_interval: u64,

    /// 数据流量收集插入数据库间隔，单位毫秒
    #[arg(long, env = "DAIRO_NPS_DATA_COLLECT_INSERT_INTERVAL", default_value = "6000")]
    pub data_collect_insert_interval: u64,

    /// 数据保存目录(数据库文件、账号文件都保存在这个目录下)
    #[arg(long, env = "DAIRO_NPS_DATA_DIR", default_value = DEFAULT_DATA_DIR)]
    pub data_dir: String,
}
