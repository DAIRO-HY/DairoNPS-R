use super::mapper_info::MapperInfo;
use super::table_info::{ColumnInfo, TableInfo};
use std::fs;
use std::path::Path;

/// 从指定目录下的SQL文件中读取表信息列表
pub fn read(dir: &str) -> Vec<TableInfo> {
    let mut tables = Vec::new();
    loop_files(Path::new(dir), &mut |file_path| {
        if file_path.extension().unwrap() != "sql" {
            return;
        }
        let Some(table) = read_table_info_from_sql(file_path) else {
            return;
        };
        tables.push(table);
    })
    .unwrap();
    tables
}

/// 从SQL文件中提取表信息
fn read_table_info_from_sql(sql_path: &Path) -> Option<TableInfo> {
    let mut sql = fs::read_to_string(sql_path).unwrap();
    sql = sql.replace("\r\n", "\n");
    sql = sql.replace("\r", "\n");

    let Some(table_name) = find_table_name(sql.clone()) else {
        return None;
    };

    let column_list = sql
        .split('\n')
        .filter_map(|line| read_column_info_from_line(line))
        .collect::<Vec<_>>();
    Some(TableInfo {
        name: table_name,
        nick: "".to_string(),
        columns: column_list,
        comment: "".to_string(),
        mappers: Vec::new(),
    })
}

// 从SQL语句中提取表名
fn find_table_name(sql: String) -> Option<String> {
    let mut sql = sql.to_string();

    // 将多个空格替换为一个空格
    while sql.contains("  ") {
        sql = sql.replace("  ", " ");
    }
    sql = sql.replace("\r\n", "\n");
    sql = sql.replace("\r", "\n");
    let Some(create_table_index) = sql.to_uppercase().find("CREATE TABLE") else {
        return None;
    };
    sql = sql[create_table_index + 12..].trim().to_string();
    if sql.starts_with("IF NOT EXISTS") {
        sql = sql[13..].trim().to_string();
    }

    let next_space_index = sql.find(' ').unwrap_or(sql.len());
    let next_newline_index = sql.find('\n').unwrap_or(sql.len());

    // 取两者中较小的索引，确保正确提取表名
    let next_index = next_space_index.min(next_newline_index);

    let table_name = &sql[..next_index];
    Some(table_name.to_string())
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
    if line.trim().to_uppercase().starts_with("CREATE "){
        return None;
    }
    if line.to_uppercase().contains("CREATE TABLE") {
        return None; // 跳过表定义行
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


/// 遍历 dir 下的所有子孙“文件”（不含目录），对每个文件调用回调 f。
fn loop_files<F>(dir: &Path, f: &mut F) -> std::io::Result<()>
where
    F: FnMut(&Path),
{
    // read_dir 返回的是 Iterator<Item = io::Result<DirEntry>>
    for entry_res in fs::read_dir(dir)? {
        let entry = entry_res?; // 传播 read_dir 内部错误
        let path = entry.path();

        // 注意：symlink_metadata 不会跟随符号链接；metadata 会跟随
        let metadata = fs::symlink_metadata(&path)?;

        if metadata.is_dir() {
            // 递归进入子目录
            loop_files(&path, f)?;
        } else if metadata.is_file() {
            // 仅对文件回调
            f(&path);
        } else {
            // 其他类型（符号链接、设备文件等），按需处理或忽略
            // 如果想把指向文件的符号链接也当作文件，可改用 metadata() 并判断 is_file()
        }
    }
    Ok(())
}
