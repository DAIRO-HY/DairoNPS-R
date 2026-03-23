use crate::extension::SelectSingleExt;
use rusqlite::Connection;
use rust_embed::RustEmbed;
use std::sync::OnceLock;
use tokio::sync::Mutex;

#[derive(RustEmbed)]
#[folder = "assets/sql/"]
struct Assets;

// VERSION 数据库版本号
const VERSION: i32 = 1;

//由于对数据操作比较少，全局一个连接池就够了
pub static CONN: OnceLock<Mutex<Connection>> = OnceLock::new();
pub async fn connection() -> tokio::sync::MutexGuard<'static, Connection> {
    CONN.get().unwrap().lock().await
}

// 初始化数据库连接和表结构
pub async fn init() {
    create_connection().await;
}

// 获取数据库连接
async fn create_connection() {
    let conn = new_connection();
    upgrade(&conn);
    conn.busy_timeout(std::time::Duration::from_millis(10000))
        .unwrap();
    CONN.set(Mutex::new(conn)).ok();
}

pub fn new_connection() -> Connection {
    rusqlite::Connection::open(crate::constant::nps_constant::SQLITE_FILE).unwrap()
}

// 数据库升级
fn upgrade(conn: &Connection) {
    let version: i32 = conn.select_single("PRAGMA USER_VERSION", []);
    if version == 0 {
        // // 设置 WAL（返回值是实际模式，可能被 SQLite 调整）
        // conn.execute_batch("PRAGMA journal_mode = WAL;").unwrap();

        // // 可选：调低同步级别（WAL 常与 NORMAL 同用以提升吞吐）
        // conn.execute_batch("PRAGMA synchronous = NORMAL;").unwrap();

        // // 可选：设置自动 checkpoint 频率（默认 1000 页）
        // conn.execute_batch("PRAGMA wal_autocheckpoint = 1000;")
        //     .unwrap();

        create_table(conn);

        //第一次创建数据库时往系统配置表插入一条数据
        //ExecIgnoreError("insert into system_config(inData, outData) values (0, 0);")
    } else {
    }

    //设置数据库版本号
    conn.execute(&(format!("PRAGMA USER_VERSION = {}", VERSION)), [])
        .unwrap();
}

// 创建表
fn create_table(conn: &Connection) {
    for sql_file in ["xxx.sql", "client.sql", "channel_data_size.sql"] {
        execute_sql_file(conn, sql_file);
    }
}

// 执行 SQL 文件中的多条 SQL 语句
fn execute_sql_file(conn: &Connection, sql_file: &str) {
    let sql_mbed = Assets::get(sql_file).unwrap();
    let sql_str = std::str::from_utf8(sql_mbed.data.as_ref()).unwrap();
    for s in sql_str.split(";") {
        if s.trim().is_empty() {
            continue;
        }
        conn.execute(s, []).unwrap();
    }
}
