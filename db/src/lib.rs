use std::error::Error;
// use crate::extension::SelectSingleExt;
use rust_embed::RustEmbed;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Sqlite, SqlitePool, Transaction};
use std::sync::LazyLock;

#[derive(RustEmbed)]
#[folder = "../nps/assets/sql/"]
struct Assets;

// VERSION 数据库版本号
const VERSION: i32 = 1;

/// 全局数据库连接池
static DB_CONN: LazyLock<SqlitePool> = LazyLock::new(|| {
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect_lazy("sqlite://dairo-nps.sqlite?mode=rwc")
        .unwrap()
});

/// 消耗一个链接
pub fn get() -> SqlitePool {
    DB_CONN.clone()
}

// 初始化数据库连接和表结构
pub async fn init() {
    eprintln!("-->数据库初始化中...");
    init_db().await.unwrap_or_else(|it| {
        eprintln!("-->数据库初始化失败: {}", it);
        std::process::exit(1);
    });
}

async fn init_db() -> Result<(), Box<dyn Error>> {
    let mut tx = get().begin().await?;

    //升级数据库（如果需要）
    upgrade(&mut tx).await?;

    // 初始化数据库
    init_data(&mut tx).await?;
    tx.commit().await?;
    Ok(())
}

// 数据库升级
async fn upgrade(tx: &mut Transaction<'_, Sqlite>) -> Result<(), Box<dyn Error>> {
    let version: i64 = sqlx::query_scalar("PRAGMA USER_VERSION")
        .fetch_one(&mut **tx)
        .await
        .unwrap_or(0);
    create_table(tx).await?;
    if version == 0 {
        // // 设置 WAL（返回值是实际模式，可能被 SQLite 调整）
        // conn.execute_batch("PRAGMA journal_mode = WAL;").unwrap();

        // // 可选：调低同步级别（WAL 常与 NORMAL 同用以提升吞吐）
        // conn.execute_batch("PRAGMA synchronous = NORMAL;").unwrap();

        // // 可选：设置自动 checkpoint 频率（默认 1000 页）
        // conn.execute_batch("PRAGMA wal_autocheckpoint = 1000;")
        //     .unwrap();

        //第一次创建数据库时往系统配置表插入一条数据
        //ExecIgnoreError("insert into system_config(inData, outData) values (0, 0);")
        sqlx::query("insert into system_config(in_len,out_len)values(0,0)")
            .execute(&mut **tx)
            .await?;
    } else {
    }

    //设置数据库版本号
    sqlx::query(&(format!("PRAGMA USER_VERSION = {}", VERSION)))
        .execute(&mut **tx)
        .await?;
    Ok(())
}

// 初始化数据库
async fn init_data(tx: &mut Transaction<'_, Sqlite>) -> Result<(), Box<dyn Error>> {
    // sqlx::query(
    //     r#"
    //     delete from client where deleted = 1;
    //     delete from channel where client_id not in (select id from client);
    //     "#
    // )
    // .execute(&mut **tx)
    // .await?;
    Ok(())
}

// 创建表
async fn create_table(tx: &mut Transaction<'_, Sqlite>) -> Result<(), Box<dyn Error>> {
    for sql_file in ["xxx.sql", "client.sql", "channel.sql", "channel_data.sql","system_config.sql"] {
        execute_sql_file(tx, sql_file).await?;
    }
    Ok(())
}

// 执行 SQL 文件中的多条 SQL 语句
async fn execute_sql_file(
    tx: &mut Transaction<'_, Sqlite>,
    sql_file: &str,
) -> Result<(), Box<dyn Error>> {
    let Some(sql_mbed) = Assets::get(sql_file) else {
        return Err(From::from(format!("{} not found", sql_file)));
    };
    let sql_str = std::str::from_utf8(sql_mbed.data.as_ref())?;
    for s in sql_str.split(";") {
        if s.trim().is_empty() {
            continue;
        }
        eprintln!("-->执行 SQL 文件: {}\n{}", sql_file, s);
        sqlx::query(s).execute(&mut **tx).await?;
    }
    Ok(())
}
