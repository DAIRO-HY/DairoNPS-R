use rust_embed::RustEmbed;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use sqlx_context::DbContext;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(RustEmbed)]
#[folder = "../DairoNPS/assets/ddl/"]
struct Assets;

// VERSION 数据库版本号
const VERSION: i32 = 1;

/// 数据库文件名
const DB_FILE_NAME: &str = "dairo-nps.sqlite";

/// 获取数据目录下的数据库文件路径
pub fn db_path(data_dir: &str) -> PathBuf {
    Path::new(data_dir).join(DB_FILE_NAME)
}


/// 全局数据库连接池
static DB_CONN: OnceLock<SqlitePool> = OnceLock::new();
// || {
//     SqlitePoolOptions::new()
//         .max_connections(5)
//         .connect_lazy("sqlite://data/farming2.sqlite?mode=rwc")
//         .unwrap()
// });

/// 消耗一个链接
pub fn get() -> SqlitePool {
    DB_CONN.get().unwrap().clone()
}

/// 消耗一个链接
pub fn get_context() -> DbContext {
    DbContext::new(DB_CONN.get().unwrap().clone())
}

// 初始化数据库连接和表结构
// - data_dir 数据目录,数据库文件保存在该目录下
pub async fn init(data_dir: &str) {
    //SQLite不会自动创建目录,先确保数据目录存在
    std::fs::create_dir_all(data_dir).unwrap();
    let db_path = db_path(data_dir);
    let db_path = db_path.to_str().unwrap();
    DB_CONN
        .set(
            SqlitePoolOptions::new()
                .max_connections(5)
                .connect_lazy(format!("sqlite://{}?mode=rwc", db_path).as_str())
                .unwrap(),
        )
        .unwrap();
    eprintln!("-->数据库初始化中...");
    init_db().await.unwrap_or_else(|it| {
        eprintln!("-->数据库初始化失败: {}", it);
        std::process::exit(1);
    });

    eprintln!("-->数据库初始化完成。");
}

async fn init_db() -> Result<(), Box<dyn Error>> {
    //升级数据库（如果需要）
    upgrade().await?;
    Ok(())
}

// 数据库升级
async fn upgrade() -> Result<(), Box<dyn Error>> {
    let mut ctx = get_context();
    let version: i64 = sqlx::query_scalar("PRAGMA USER_VERSION")
        .fetch_one(&mut ctx)
        .await
        .unwrap_or(0);

    //创建表
    create_table(&mut ctx).await?;
    if version < 1{

        //第一次创建数据库时往系统配置表插入一条数据
        sqlx::query("insert into system_config(in_len,out_len)values(0,0)")
            .execute(&mut ctx)
            .await?;
    }

    //设置数据库版本号(版本号没变时不写库,否则build.rs监听的数据库文件每次都会变,导致每次编译都重跑build.rs)
    if version != VERSION as i64 {
        sqlx::query(&(format!("PRAGMA USER_VERSION = {}", VERSION)))
            .execute(&mut ctx)
            .await?;
    }
    Ok(())
}

// 创建表(按外键依赖顺序排列)
async fn create_table(ctx: &mut DbContext) -> Result<(), Box<dyn Error>> {
    for sql_file in [
        "client.sql",
        "channel.sql",
        "traffic_stats.sql",
        "system_config.sql",
        "forward.sql",
    ] {
        execute_sql_file(ctx, sql_file).await?;
    }
    Ok(())
}

/// 执行 SQL 文件中的多条 SQL 语句
async fn execute_sql_file(ctx: &mut DbContext, sql_file: &str) -> Result<(), Box<dyn Error>> {
    let Some(sql_mbed) = Assets::get(sql_file) else {
        return Err(From::from(format!("{} not found", sql_file)));
    };
    let sql_str = std::str::from_utf8(sql_mbed.data.as_ref())?;
    for s in sql_str.split(";") {
        if s.trim().is_empty() {
            continue;
        }
        // eprintln!("-->执行 SQL 文件: {}\n{}", sql_file, s);
        sqlx::query(s).execute(&mut *ctx).await?;
    }
    Ok(())
}
