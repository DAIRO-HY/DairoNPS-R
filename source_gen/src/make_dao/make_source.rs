use super::table_info::TableInfo;
use std::env;
use std::fs;
use std::path::Path;
use syn;

pub fn make(tables: Vec<TableInfo>) {
    //生成的dao源代码写到 OUT_DIR/dao 目录
    let save_path = Path::new(&(env::var("OUT_DIR").unwrap().as_str())).join("dao");
    if !save_path.exists() {
        //如果目录不存在则创建
        fs::create_dir(&save_path).unwrap();
    }

    tables.iter().for_each(|table| {
        let mut dao_src = String::new();

        // dao_src.push_str(&table.make_query_entity_src());

        //生成实体类的源代码
        dao_src.push_str(&table.make_entity_src());

        //生成插入函数的源代码
        dao_src.push_str(&table.make_insert_func());

        //生成查询函数的源代码
        dao_src.push_str(&table.make_select_one_func());

        //生成查询所有数据的函数的源代码
        dao_src.push_str(&table.make_select_all_func());

        //生成更新函数的源代码
        dao_src.push_str(&table.make_update_func());

        //生成删除函数的源代码
        dao_src.push_str(&table.make_set_delete_func());

        //生成删除忽略版本函数的源代码
        dao_src.push_str(&table.make_set_delete_ignore_version_func());

        //生成物理删除函数的源代码
        dao_src.push_str(&table.make_delete_func());

        // 生成mapper函数的源代码
        dao_src.push_str(&table.make_mapper_func());

        dao_src.insert_str(
            0,
            &format!("// Generated at {}\n", chrono::Local::now().to_rfc3339()),
        );
        let file_path = &save_path.join(format!("{}_dao.rs", table.name));

        // 解析成 syn AST
        let rust_src = syn::parse_str(dao_src.as_str()).unwrap_or_else(|it|{
            eprintln!("cargo:warning=解析{}文件出错:{}", file_path.display(), it);
            eprintln!("cargo:warning=文件内容:\n{}", dao_src);
            panic!("文件解析失败");
        });

        // 使用 prettyplease 格式化
        let formatted_rust_src = prettyplease::unparse(&rust_src);
        fs::write(&file_path, formatted_rust_src).unwrap();

        // let file_path = Path::new("/Users/zhoulq/dev/rust/DairoNPS-R/nps/src/").join(format!("{}_dao.rs", table.name));
        // fs::write(&file_path, dao_src).unwrap();
        println!(
            "cargo:warning=Generated DAO source for table '{}' at '{}'",
            table.name,
            file_path.display()
        );
    });
}
