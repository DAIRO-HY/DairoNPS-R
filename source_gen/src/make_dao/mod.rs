pub mod make_source;
pub mod read_table_from_sql;
pub mod table_info;
pub mod mapper_info;

use mapper_info::MapperInfo;
use std::path::Path;
use std::fs;

pub fn make(dir: &str, mapper_dir: &str) {
    let mut tables = read_table_from_sql::read(dir);
    tables.iter_mut().for_each(|table| {
        table.mappers = read_mapper(&table.name, mapper_dir);
    });
    make_source::make(tables);
}

/// 从 mapper_dir 目录下读取与表名对应的 mapper 文件，提取映射信息列表
fn read_mapper(table: &str, mapper_dir: &str) -> Vec<MapperInfo> {
    let mut mappers = Vec::new();
    let mapper_path = Path::new(mapper_dir).join(format!("{}.mapper", table));
    if !mapper_path.exists() {
        return mappers;
    }
    let content = fs::read_to_string(mapper_path).unwrap();
    let mut lines = content.lines();

    let mut current_mapper = MapperInfo::default();
    while let Some(line) = lines.next() {
        if line.trim().starts_with("fn ") {
            current_mapper.func = line.trim().to_string();
            mappers.push(current_mapper);
            current_mapper = MapperInfo::default();
            continue;
        }
        current_mapper.sql.push_str(line);
    }
    mappers
}