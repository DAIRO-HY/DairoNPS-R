use crate::utils;
use serde::Serialize;
use super::mapper_info::MapperInfo;

/// 数据库表信息
#[derive(Debug, Serialize)]
pub struct TableInfo {
    // 表名
    pub name: String,

    // 表昵称
    pub nick: String,

    // 列信息列表
    pub columns: Vec<ColumnInfo>,

    // 表注释
    pub comment: String,

    // 映射信息列表
    pub mappers: Vec<MapperInfo>,
}

impl TableInfo {
    /// 判断是否有version列
    pub fn has_version(&self) -> bool {
        self.columns.iter().any(|it| it.name == "version")
    }

    /// 判断是否有deleted列
    pub fn has_deleted(&self) -> bool {
        self.columns.iter().any(|it| it.name == "deleted")
    }

    /// 判断是否有deleted_at列
    pub fn has_deleted_at(&self) -> bool {
        self.columns.iter().any(|it| it.name == "deleted_at")
    }

    /// 判断是否有deleted_by列
    pub fn has_deleted_by(&self) -> bool {
        self.columns.iter().any(|it| it.name == "deleted_by")
    }

    /// 获取主键列列表
    fn primary_key_columns(&self) -> impl Iterator<Item = &ColumnInfo> {
        self.columns.iter().filter(|it| it.is_primary_key)
    }

    /// 获取实体类名称
    pub fn get_entity_name(&self) -> String {
        utils::snake_to_pascal(&self.name, "_")
    }

    /// 生成实体类的源代码
    pub fn make_entity_src(&self) -> String {
        let mut entity_src = String::new();
        entity_src.push_str("#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]\n");
        entity_src.push_str(&format!(
            "pub struct {} {{\n",
            self.get_entity_name()
        ));
        self.columns.iter().for_each(|column| {
            let mut field_type = Self::map_data_type_to_rust_type(&column.data_type).to_string();
            if column.is_nullable {
                field_type = format!("Option<{}>", field_type);
            }
            entity_src.push_str(&format!("    pub {}: {},\n", column.name, field_type));
        });
        entity_src.push_str("}\n");
        entity_src
    }

    /// 生成分页查询条件实体类的源代码
    pub fn make_query_entity_src(&self) -> String {
        let mut entity_src = String::new();
        entity_src.push_str("#[derive(Default)]\n");
        entity_src.push_str(&format!(
            "pub struct {}Query {{\n",
            utils::snake_to_pascal(&self.name, "_")
        ));
        self.columns.iter().for_each(|it| {
            let mut field_type = Self::map_data_type_to_rust_type(&it.data_type).to_string();
            field_type = format!("QueryModel<{}>", field_type);
            entity_src.push_str(&format!("    pub {}: {},\n", it.name, field_type));
            // match it.name.as_str() {
            //     "created_at" | "updated_at" => {
            //         entity_src.push_str(&format!("    pub {}_start: Option<i64>,\n", it.name));
            //         entity_src.push_str(&format!("    pub {}_end: Option<i64>,\n", it.name));
            //     }
            //     _ => {}
            // }
        });
        entity_src.push_str("}\n");
        entity_src
    }

    /// 将数据库数据类型映射为Rust类型
    fn map_data_type_to_rust_type(data_type: &str) -> &str {
        match data_type.to_uppercase().as_str() {
            "INTEGER" | "INT" => "i64",
            "BIGINT" => "i64",
            "INT8" => "i8",
            "INT16" => "i16",
            "INT32" => "i32",
            "INT64" => "i64",
            "VARCHAR" | "TEXT" => "String",
            "BOOLEAN" => "bool",
            "FLOAT" => "f32",
            "DOUBLE" => "f64",
            _ => "String", // 默认使用String类型
        }
    }

    // 获取自增且主键的列名
    fn auto_increment_and_primary_column(&self) -> Option<String> {
        self.columns.iter().find_map(|column| {
            if column.is_auto_increment && column.is_primary_key {
                Some(column.name.clone())
            } else {
                None
            }
        })
    }

    // fn get_insert_columns(&self) -> Vec<&ColumnInfo> {
    //     self.columns
    //         .iter()
    //         .filter(|it| it.name != "version" && it.name != "created_at" && it.name != "updated_at")
    //         .collect()
    // }

    /// 生成插入函数的源代码
    pub fn make_insert_func(&self) -> String {
        let mut insert_columns: Vec<&str> = Vec::new(); // 构建插入列列表
        let mut insert_params_replace: Vec<&str> = Vec::new(); // 构建插入参数占位符列表
        let mut insert_params: Vec<String> = Vec::new(); // 构建插入参数列表

        let mut need_now = false; // 是否需要生成获取当前时间的代码
        self.columns
            .iter()
            .filter(|it| {
                if it.is_auto_increment && it.is_primary_key {
                    return false;
                }
                if it.name == "deleted_at" {
                    return false;
                }
                if it.name == "deleted_by" {
                    return false;
                }
                if it.name == "deleted" {
                    return false;
                }
                if it.name == "version" {
                    return false;
                }
                true
            })
            .for_each(|it| {
                insert_columns.push(it.name.as_str());
                insert_params_replace.push("?");
                if it.name == "created_at" {
                    need_now = true;
                    insert_params.push("timestamp".to_string());
                } else if it.name == "updated_at" {
                    need_now = true;
                    insert_params.push("timestamp".to_string());
                } else {
                    insert_params.push(format!("entity.{}", it.name));
                }
            });

        // 构建插入SQL语句
        let mut insert_sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.name,
            insert_columns.join(", "),
            insert_params_replace.join(", ")
        );

        let fn_template = if let Some(key) = self.auto_increment_and_primary_column() {
            insert_sql.push_str(&format!(" RETURNING {}", key));
            r##"
            /// 插入数据
            pub fn insert(conn: &rusqlite::Connection, entity: [ENTITY]) -> rusqlite::Result<i64,rusqlite::Error> {
                [TIME_CODE]
                const SQL: &str = "[SQL];";
                conn.query_row(SQL, ([PARAM]), |it|it.get(0))
            }"##
        } else {
            r##"
            pub fn insert(conn: &rusqlite::Connection, entity: [ENTITY]) -> Option<rusqlite::Error> {
                [TIME_CODE]
                const SQL: &str = "[SQL];";
                if let Err(e) = conn.execute(
                    SQL,
                    ([PARAM]),
                ) {
                    return Some(e);
                }
                None
            }"##
        };

        let time_code = if need_now {
            "let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;"
        } else {
            ""
        };
        fn_template
            .replace("[TIME_CODE]", time_code)
            .replace("[SQL]", &insert_sql)
            .replace("[PARAM]", &(insert_params.join(", ")))
            .replace("[ENTITY]", &utils::snake_to_pascal(&self.name, "_"))
    }

    /// 生成查询函数的源代码
    pub fn make_select_one_func(&self) -> String {
        let mut where_params: Vec<&str> = Vec::new(); // 构建插入参数列表
        let mut func_params: Vec<String> = Vec::new(); // 构建函数参数列表

        // 构建WHERE条件列列表
        let where_columns: Vec<String> = self
            .primary_key_columns()
            .map(|it| {
                where_params.push(it.name.as_str());
                func_params.push(format!(
                    "{}:{}",
                    it.name,
                    Self::map_data_type_to_rust_type(&it.data_type)
                ));
                format!("{} = ?", it.name)
            })
            .collect();
        if where_columns.is_empty() {
            // 如果没有主键列则不生成查询函数
            return String::new();
        }

        // 要查询的字段列表，默认查询所有非TEXT和BLOB类型的字段以避免性能问题
        let filed_columns: Vec<&str> = self.columns.iter().map(|it| it.name.as_str()).collect();

        // 构建查询参数与列的映射列表
        let field_mappings: Vec<String> = filed_columns
            .iter()
            .enumerate()
            .map(|(i, it)| format!("{}: row.get({})?", it, i))
            .collect();

        // 生成查询一条数据的函数的源代码
        let select_one_template = r##"

        /// 通过主键查询一条数据
        pub fn select_one(conn: &rusqlite::Connection, [FUNC_PARAMS]) -> rusqlite::Result<[ENTITY], rusqlite::Error> {
            const SQL: &str = "SELECT [FIELDS] FROM [TABLE] WHERE [WHERE];";
            let mut stmt = conn.prepare(SQL)?;
            stmt.query_one(rusqlite::params!([PARAM]), |row| {
                Ok([ENTITY] {
                    [FIELD_MAPPINGS]
                })
            })
        }
        "##;
        select_one_template
            .replace("[FUNC_PARAMS]", &func_params.join(", "))
            .replace("[TABLE]", &self.name)
            .replace("[FIELDS]", &filed_columns.join(", "))
            .replace("[ENTITY]", &utils::snake_to_pascal(&self.name, "_"))
            .replace("[WHERE]", &where_columns.join(" AND "))
            .replace("[PARAM]", &where_params.join(", "))
            .replace("[FIELD_MAPPINGS]", &field_mappings.join(", "))
    }

    /// 生成查询函数的源代码
    pub fn make_select_all_func(&self) -> String {
        // 要查询的字段列表，默认查询所有非TEXT和BLOB类型的字段以避免性能问题
        let filed_columns: Vec<&str> = self
            .columns
            .iter()
            .filter_map(|it| {
                let ty = it.data_type.to_uppercase();
                println!("filed_type: {}->{}", ty, ty == "TEXT" || ty == "BLOB");
                if ty == "TEXT" || ty == "BLOB" {
                    // 如果是TEXT或BLOB类型则不生成查询函数以避免性能问题
                    return None;
                }
                Some(it.name.as_str())
            })
            .collect();

        println!("filed_columns: {:?}", filed_columns);

        // 构建查询参数与列的映射列表
        let field_mappings: Vec<String> = filed_columns
            .iter()
            .enumerate()
            .map(|(i, it)| format!("{}: row.get({})?", it, i))
            .collect();
        let mut templates = Vec::new();
        if self.has_deleted() {
            // 如果有deleted列则生成两个查询函数一个查询所有未删除数据一个查询所有数据以满足不同的查询需求

            // 生成查询所有未删除数据的函数的源代码
            let select_all_template = r##"
                /// 查询所有未删除的数据
                pub fn select_all(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<[ENTITY]>, rusqlite::Error> {
                    const SQL: &str = "SELECT [FIELDS] FROM [TABLE] WHERE deleted = 0;";
                    let mut stmt = conn.prepare(SQL)?;
                    stmt.query_map([], |row| {
                        Ok([ENTITY] {
                            [FIELD_MAPPINGS],..Default::default()
                        })
                    })?.collect()
                }
                "##;

            // 生成查询所有数据的函数的源代码
            let select_all_include_deleted_template = r##"
                /// 查询所有数据，包括已删除的数据
                pub fn select_all_include_deleted(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<[ENTITY]>, rusqlite::Error> {
                    const SQL: &str = "SELECT [FIELDS] FROM [TABLE];";
                    let mut stmt = conn.prepare(SQL)?;
                    stmt.query_map([], |row| {
                        Ok([ENTITY] {
                            [FIELD_MAPPINGS],..Default::default()
                        })
                    })?.collect()
                }
                "##;
            templates.push(select_all_template);
            templates.push(select_all_include_deleted_template);
        } else {
            templates.push(r##"
        pub fn select_all(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<[ENTITY]>, rusqlite::Error> {
            const SQL: &str = "SELECT [FIELDS] FROM [TABLE]";
            let mut stmt = conn.prepare(SQL)?;
            stmt.query_map([], |row| {
                Ok([ENTITY] {
                    [FIELD_MAPPINGS],..Default::default()
                })
            })?.collect()
        }
        "##);
        }

        templates
            .iter()
            .map(|it| -> String {
                it.replace("[TABLE]", &self.name)
                    .replace("[FIELDS]", &filed_columns.join(", "))
                    .replace("[ENTITY]", &utils::snake_to_pascal(&self.name, "_"))
                    .replace("[FIELD_MAPPINGS]", &field_mappings.join(", "))
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 生成更新函数的源代码
    pub fn make_update_func(&self) -> String {
        let mut update_columns: Vec<String> = Vec::new(); // 构建更新列列表
        let mut update_params: Vec<String> = Vec::new(); // 构建更新参数列表
        let mut where_columns: Vec<String> = Vec::new(); // 构建WHERE条件列列表

        let has_version = self.has_version();
        if has_version {
            // 如果有version列则在更新时自动加1
            update_columns.push("version = version + 1".to_string());
        }
        self.columns
            .iter()
            .filter(|it| {
                if it.is_primary_key {
                    return false;
                }
                if it.name == "created_at" {
                    return false;
                }
                if it.name == "deleted_at" {
                    return false;
                }
                if it.name == "deleted" {
                    return false;
                }
                if it.name == "version" {
                    return false;
                }
                true
            })
            .for_each(|it| {
                update_columns.push(format!("{} = ?", it.name));
                if it.name == "updated_at" {
                    update_params.push("std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64".to_string());
                } else {
                    update_params.push(format!("entity.{}", it.name));
                }
            });

        // 构建WHERE条件列列表，默认使用主键列作为更新条件
        self.columns
            .iter()
            .filter(|it| it.is_primary_key)
            .for_each(|it| {
                where_columns.push(format!("{} = ?", it.name));
                update_params.push(format!("entity.{}", it.name));
            });
        if has_version {
            // 如果有version列则在WHERE条件中加入version列以实现乐观锁
            where_columns.push("version = ?".to_string());
            update_params.push("entity.version".to_string());
        }
        if self.has_deleted() {
            // 如果有deleted列则在WHERE条件中加入deleted = 0以避免更新已删除的数据
            where_columns.push("deleted = 0".to_string());
        }

        // 构建更新SQL语句
        let update_sql = format!(
            "UPDATE {} SET {} WHERE {}",
            self.name,
            update_columns.join(", "),
            where_columns.join(" AND ")
        );

        if where_columns.is_empty() {
            // 如果没有WHERE条件则不生成更新函数以避免误操作
            return String::new();
        }

        r##"
        /// 更新数据
        pub fn update(conn: &rusqlite::Connection, entity: [ENTITY]) -> Option<rusqlite::Error> {
            const SQL: &str = "[SQL];";
            match conn.execute(
                SQL,
                rusqlite::params!([PARAM]),
            ) {
                Ok(count) => {
                    if count == 0 {
                        return Some(rusqlite::Error::QueryReturnedNoRows);
                    }
                }
                Err(e) => return Some(e),
            }
            None
        }
        "##
        .replace("[SQL]", &update_sql)
        .replace("[PARAM]", &(update_params.join(", ")))
        .replace("[ENTITY]", &utils::snake_to_pascal(&self.name, "_"))
    }

    /// 生成删除函数的源代码
    pub fn make_delete_func(&self) -> String {
        if !self.has_deleted() {
            return String::new();
        }
        let mut where_columns = Vec::new(); // 构建WHERE条件列列表
        let mut sql_params: Vec<&str> = Vec::new(); // 构建删除参数列表
        let mut func_params: Vec<String> = Vec::new(); // 构建函数参数列表
        let mut update_fields = vec!["deleted = 1"]; // 构建更新字段列表
        self.primary_key_columns().for_each(|it| {
            func_params.push(format!(
                "{}: {}",
                it.name,
                Self::map_data_type_to_rust_type(&it.data_type)
            ));
            where_columns.push(format!("{} = ?", it.name));
        });
        if where_columns.is_empty() {
            // 如果没有主键列则不生成删除函数以避免误操作
            return String::new();
        }

        if self.has_version() {
            // 如果有version列则在删除时自动加1以实现乐观锁
            func_params.push("version: i64".to_string());
            where_columns.push("version = ?".to_string());
            update_fields.push("version = version + 1");
        }
        if self.has_deleted_at() {
            // 如果有deleted_at列则在删除时自动设置删除时间
            update_fields.push("deleted_at = ?");
        }
        if self.has_deleted_by() {
            // 如果有deleted_by列则在删除时自动设置删除者
            update_fields.push("deleted_by = ?");
            func_params.push("deleted_by: String".to_string());
        }
        where_columns.push("deleted = 0".to_string());

        //------------------------------按照参数顺序添加------------------------------//
        if self.has_deleted_at() {
            // 如果有deleted_at列则在删除时自动设置删除时间
            sql_params.push("std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64");
        }
        if self.has_deleted_by() {
            // 如果有deleted_by列则在删除时自动设置删除者
            sql_params.push("deleted_by");
        }
        self.primary_key_columns().for_each(|it| {
            sql_params.push(it.name.as_str());
        });
        if self.has_version() {
            // 如果有version列则在删除时自动加1以实现乐观锁
            sql_params.push("version");
        }

        let delete_sql = format!(
            "UPDATE {} SET {} WHERE {}",
            self.name,
            update_fields.join(", "),
            where_columns.join(" AND ")
        );
        r##"
        /// 逻辑删除数据
        pub fn delete(conn: &rusqlite::Connection, [FUNC_PARAMS]) -> Option<rusqlite::Error> {
            const SQL: &str = "[SQL];";
            match conn.execute(
                SQL,
                rusqlite::params!([PARAM]),
            ) {
                Ok(count) => {
                    if count == 0 {
                        return Some(rusqlite::Error::QueryReturnedNoRows);
                    }
                }
                Err(e) => return Some(e),
            }
            None
        }
        "##
        .replace("[SQL]", &delete_sql)
        .replace("[PARAM]", &sql_params.join(", "))
        .replace("[FUNC_PARAMS]", &func_params.join(", "))
    }

    /// 生成删除忽略版本函数的源代码
    pub fn make_delete_ignore_version_func(&self) -> String {
        if !self.has_deleted() {
            return String::new();
        }
        let mut where_columns = Vec::new(); // 构建WHERE条件列列表
        let mut sql_params: Vec<&str> = Vec::new(); // 构建删除参数列表
        let mut func_params: Vec<String> = Vec::new(); // 构建函数参数列表
        let mut update_fields = vec!["deleted = 1"]; // 构建更新字段列表
        self.primary_key_columns().for_each(|it| {
            func_params.push(format!(
                "{}: {}",
                it.name,
                Self::map_data_type_to_rust_type(&it.data_type)
            ));
            where_columns.push(format!("{} = ?", it.name));
        });
        if where_columns.is_empty() {
            // 如果没有主键列则不生成删除函数以避免误操作
            return String::new();
        }

        if self.has_deleted_at() {
            // 如果有deleted_at列则在删除时自动设置删除时间
            update_fields.push("deleted_at = ?");
        }
        if self.has_deleted_by() {
            // 如果有deleted_by列则在删除时自动设置删除者
            update_fields.push("deleted_by = ?");
            func_params.push("deleted_by: String".to_string());
        }
        where_columns.push("deleted = 0".to_string());

        //------------------------------按照参数顺序添加------------------------------//
        if self.has_deleted_at() {
            // 如果有deleted_at列则在删除时自动设置删除时间
            sql_params.push("std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64");
        }
        if self.has_deleted_by() {
            // 如果有deleted_by列则在删除时自动设置删除者
            sql_params.push("deleted_by");
        }
        self.primary_key_columns().for_each(|it| {
            sql_params.push(it.name.as_str());
        });

        let delete_sql = format!(
            "UPDATE {} SET {} WHERE {}",
            self.name,
            update_fields.join(", "),
            where_columns.join(" AND ")
        );
        r##"
        /// 忽略版本号逻辑删除数据
        pub fn delete_ignore_version(conn: &rusqlite::Connection, [FUNC_PARAMS]) -> Option<rusqlite::Error> {
            const SQL: &str = "[SQL];";
            match conn.execute(
                SQL,
                rusqlite::params!([PARAM]),
            ) {
                Ok(count) => {
                    if count == 0 {
                        return Some(rusqlite::Error::QueryReturnedNoRows);
                    }
                }
                Err(e) => return Some(e),
            }
            None
        }
        "##
        .replace("[SQL]", &delete_sql)
        .replace("[PARAM]", &sql_params.join(", "))
        .replace("[FUNC_PARAMS]", &func_params.join(", "))
    }

    /// 生成删除忽略版本函数的源代码
    pub fn make_delete_ignore_version_func123(&self) -> String {
        if !self.has_deleted() {
            return String::new();
        }
        let mut where_columns = Vec::new(); // 构建WHERE条件列列表
        let mut sql_params: Vec<&str> = Vec::new(); // 构建删除参数列表
        let mut func_params: Vec<String> = Vec::new(); // 构建函数参数列表
        let mut update_fields = vec!["deleted = 1"]; // 构建更新字段列表
        self.primary_key_columns().for_each(|it| {
            func_params.push(format!(
                "{}: {}",
                it.name,
                Self::map_data_type_to_rust_type(&it.data_type)
            ));
            where_columns.push(format!("{} = ?", it.name));
        });
        if where_columns.is_empty() {
            // 如果没有主键列则不生成删除函数以避免误操作
            return String::new();
        }
        where_columns.push("deleted = 0".to_string());
        if self.columns.iter().any(|it| it.name == "deleted_at") {
            // 如果有deleted_at列则在删除时自动设置删除时间
            update_fields.push("deleted_at = ?");
            sql_params.push("std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64");
        }
        if self.columns.iter().any(|it| it.name == "deleted_by") {
            // 如果有deleted_by列则在删除时自动设置删除者
            update_fields.push("deleted_by = ?");
            sql_params.push("deleted_by");
            func_params.push("deleted_by: String".to_string());
        }

        //最后添加主键参数以保证参数顺序正确
        self.primary_key_columns().for_each(|it| {
            sql_params.push(it.name.as_str());
        });

        let delete_sql = format!(
            "UPDATE {} SET {} WHERE {}",
            self.name,
            update_fields.join(", "),
            where_columns.join(" AND ")
        );
        r##"
        /// 忽略版本号逻辑删除数据
        pub fn delete_ignore_version(conn: &rusqlite::Connection, [FUNC_PARAMS]) -> Option<rusqlite::Error> {
            const SQL: &str = "[SQL];";
            match conn.execute(
                SQL,
                ([PARAM]),
            ) {
                Ok(count) => {
                    if count == 0 {
                        return Some(rusqlite::Error::QueryReturnedNoRows);
                    }
                }
                Err(e) => return Some(e),
            }
            None
        }
        "##
        .replace("[SQL]", &delete_sql)
        .replace("[PARAM]", &sql_params.join(", "))
        .replace("[FUNC_PARAMS]", &func_params.join(", "))
    }

    /// 生成物理删除函数的源代码
    pub fn make_purge_func(&self) -> String {
        let mut where_columns = Vec::new(); // 构建WHERE条件列列表
        let mut sql_params: Vec<&str> = Vec::new(); // 构建删除参数列表
        let mut func_params: Vec<String> = Vec::new(); // 构建函数参数列表
        self.columns
            .iter()
            .filter(|it| it.is_primary_key)
            .for_each(|it| {
                sql_params.push(it.name.as_str());
                func_params.push(format!(
                    "{}: {}",
                    it.name,
                    Self::map_data_type_to_rust_type(&it.data_type)
                ));
                where_columns.push(format!("{} = ?", it.name));
            });
        if where_columns.is_empty() {
            // 如果没有主键列则不生成删除函数以避免误操作
            return String::new();
        }
        let delete_sql = format!(
            "DELETE FROM {} WHERE {}",
            self.name,
            where_columns.join(" AND ")
        );
        r##"

        /// 物理删除数据
        pub fn purge(conn: &rusqlite::Connection, [FUNC_PARAMS]) -> Option<rusqlite::Error> {
            const SQL: &str = "[SQL];";
            match conn.execute(
                SQL,
                rusqlite::params!([PARAM]),
            ) {
                Ok(count) => {
                    if count == 0 {
                        return Some(rusqlite::Error::QueryReturnedNoRows);
                    }
                }
                Err(e) => return Some(e),
            }
            None
        }
        "##
        .replace("[SQL]", &delete_sql)
        .replace("[PARAM]", &sql_params.join(", "))
        .replace("[FUNC_PARAMS]", &func_params.join(", "))
    }

    /// 生成映射函数的源代码
    pub fn make_mapper_func(&self) -> String {
        let mut func_src = String::new();
        self.mappers.iter().for_each(|mapper| {
            func_src.push_str(&mapper.make_mapper_source(self));
        });
        func_src
    }
}

/// 数据库列信息
#[derive(Debug, Serialize)]
pub struct ColumnInfo {
    // 列名
    pub name: String,
    // 列昵称
    pub nick: String,
    // 数据类型
    pub data_type: String,
    // 是否主键
    pub is_primary_key: bool,
    // 是否自增
    pub is_auto_increment: bool,
    // 默认值
    pub default_value: Option<String>,
    // 是否可为空
    pub is_nullable: bool,
    // 列注释
    pub comment: String,
}
