use super::table_info::{ColumnInfo, TableInfo};
use sqlx::{Connection, Row, SqliteConnection};

/// 从指定目录下的SQL文件中读取表信息列表
pub async fn read(url: &str) -> Vec<TableInfo> {
    let mut conn = SqliteConnection::connect(url).await.unwrap();
    sqlx::query("SELECT name,sql FROM sqlite_master WHERE type = 'table' and name NOT LIKE 'sqlite_%'")
            .fetch_all(&mut conn)
            .await
            .unwrap()
            .iter()
            .map(|it| {
                let sql: String = it.get("sql");
                let table_info = read_table_info_from_sql(sql);
                table_info
            })
            .collect()
}

/// 从建表语句的SQL中提取表信息
fn read_table_info_from_sql(sql: String) -> TableInfo {
    let mut sql = sql.replace("\r\n", "\n");
    sql = sql.replace("\r", "\n");
    let (name, comment) = find_table_name(sql.clone());

    let open_kh_index = sql.find('(').unwrap(); //寻找第一个括号索引
    let close_kh_index = sql.rfind(')').unwrap(); //寻找最后一个括号索引
    let columns_sql = sql[open_kh_index + 1..close_kh_index].to_string();
    let columns = columns_sql
        .split('\n')
        .filter_map(|line| read_column_info_from_line(line))
        .collect::<Vec<_>>();
    TableInfo {
        name,
        nick: "".to_string(),
        columns,
        comment,
        mappers: Vec::new(),
    }
}

// 从SQL语句中提取表名和表注释
fn find_table_name(sql: String) -> (String, String) {
    let mut sql = sql.to_string();

    // 将多个空格替换为一个空格
    while sql.contains("  ") {
        sql = sql.replace("  ", " ");
    }
    sql = sql.replace("\r\n", "\n");
    sql = sql.replace("\r", "\n");
    let create_table_index = sql.to_uppercase().find("CREATE TABLE").unwrap();
    sql = sql[create_table_index + 12..].trim().to_string();
    if sql.starts_with("IF NOT EXISTS") {
        sql = sql[13..].trim().to_string();
    }

    let open_kh_index = sql.find('(').unwrap(); //寻找第一个括号索引
    let block_sql = sql[..open_kh_index].to_string();
    let comment_index = block_sql.find("--"); //寻找注释标记

    let mut comment = String::new();
    let name = if let Some(comment_index) = comment_index {
        comment = block_sql[comment_index..].trim().to_string();
        block_sql[..comment_index].trim().to_string()
    } else {
        block_sql.trim().to_string()
    };
    (name, comment)
}

/// 从SQL语句的列定义行中提取列信息
fn read_column_info_from_line(line: &str) -> Option<ColumnInfo> {
    let line = line.to_string();

    // 提取注释
    let comment_index = line.find("--");
    let (line, comment) = if let Some(index) = comment_index {
        (
            line[..index].trim().to_string(),
            line[index + 2..].trim().to_string(),
        )
    } else {
        (line.trim().to_string(), "".to_string())
    };
    let mut line = line;
    while line.contains("  ") {
        line = line.replace("  ", " ");
    }

    // 提取列名
    let next_space_index = line.find(' ').unwrap_or(line.len());
    let column_name = &line[..next_space_index];
    if column_name.is_empty() {
        return None;
    }

    // 列名只能包含字母、数字和下划线
    if !column_name
        .replace("_", "")
        .chars()
        .all(|c| c.is_ascii_alphanumeric())
    {
        return None;
    }

    // 提取数据类型
    let line = line[next_space_index..].trim().to_string();
    let next_space_index = line
        .find(' ')
        .unwrap_or(line.len())
        .min(line.find(',').unwrap_or(line.len()));
    let data_type = &line[..next_space_index];

    let line = line[next_space_index..].trim().to_string().to_uppercase();
    let is_primary_key = line.contains("PRIMARY KEY"); // 判断是否包含 PRIMARY KEY
    let is_nullable = !line.contains("NOT NULL"); // 判断是否包含 NOT NULL
    let is_auto_increment = line.contains("AUTOINCREMENT") || line.contains("AUTO_INCREMENT"); // 判断是否包含 AUTOINCREMENT 或 AUTO_INCREMENT

    Some(ColumnInfo {
        name: column_name.to_string(),
        nick: "".to_string(),
        data_type: data_type.to_string(),
        is_primary_key,
        default_value: None,
        is_nullable,
        is_auto_increment,
        comment,
    })
}