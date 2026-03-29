use std::fs;
use std::path::Path;
use crate::make_dao::mapper_info::MapperInfo;
use crate::make_dao::mapper_info::CrudType;

/// 从 mapper_dir 目录下读取与表名对应的 mapper 文件，提取映射信息列表
pub fn read(table: &str, mapper_dir: &str) -> Vec<MapperInfo> {
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
            let fn_index = line.find("fn ").unwrap();

            //函数括号开始位置
            let kh_open_index = line.find('(').expect("这不是一个有效的函数格式");

            //函数括号结束位置
            let kh_close_index = line.find(')').expect("这不是一个有效的函数格式");

            //得到函数名称
            let func_name = line[fn_index + 3..kh_open_index].trim().to_string();

            //得到函数参数列表
            let func_params = line[kh_open_index + 1..kh_close_index]
                .trim()
                .to_string();

            let mut is_list = false;

            //函数返回值标记位置
            let return_type = if let Some(return_flag_index) = line.find("->") {
                let return_str = line[return_flag_index + 2..].trim();
                if return_str.starts_with("Vec<") && return_str.ends_with('>') {
                    is_list = true;
                    &return_str[4..return_str.len() - 1]
                } else {
                    return_str
                }
            } else {
                ""
            };
            current_mapper.is_list = is_list;
            current_mapper.name = func_name;
            current_mapper.params_str = func_params;
            current_mapper.return_type = return_type.to_string();

            let sql = current_mapper.sql.trim().to_uppercase();
            if sql.starts_with("SELECT"){
                current_mapper.crud_type = CrudType::Read
            } else if sql.starts_with("INSERT") {
                current_mapper.crud_type = CrudType::Create
            } else if sql.starts_with("UPDATE") {
                current_mapper.crud_type = CrudType::Update
            } else if sql.starts_with("DELETE") {
                current_mapper.crud_type = CrudType::Delete
            }else {
                current_mapper.crud_type = CrudType::Unknown
            }

            mappers.push(current_mapper);

            current_mapper = MapperInfo::default();
            continue;
        }
        if !line.trim().starts_with("//") {
            current_mapper.sql.push_str(line);
            current_mapper.sql.push('\n');
        }
    }
    mappers
}