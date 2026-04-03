use crate::make_dao::table_info::{ColumnInfo, TableInfo};
use serde::Serialize;
// use sqlparser::ast::{Expr, SelectItem, Statement};
// use sqlparser::dialect::GenericDialect;
// use sqlparser::parser::Parser;
use sqlx::{Column, Executor, SqliteConnection, Statement, TypeInfo};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize)]
pub enum CrudType {
    #[default]
    Unknown,
    Create,
    Read,
    Update,
    Delete,
}

#[derive(Debug, Default, Serialize)]
pub struct MapperInfo {
    pub sql: String,
    pub name: String,
    pub is_list: bool,
    pub return_type: String,
    pub params_str: String,
    pub crud_type: CrudType,
}

impl MapperInfo {
    /// 从 SQL 字符串中提取参数名称列表
    fn extract_params(sql: &str) -> Vec<String> {
        let bytes = sql.as_bytes();
        let mut result = Vec::new();
        let mut i = 0;

        while i < bytes.len() {
            if bytes[i] == b':' {
                let start = i + 1;
                let mut end = start;

                while end < bytes.len() {
                    let c = bytes[end];
                    if (c >= b'a' && c <= b'z')
                        || (c >= b'A' && c <= b'Z')
                        || (c >= b'0' && c <= b'9')
                        || c == b'_'
                    {
                        end += 1;
                    } else {
                        break;
                    }
                }

                if end > start {
                    result.push(sql[start..end].to_string());
                    i = end;
                    continue;
                }
            }
            i += 1;
        }
        result
    }

    // /// 从 SQL 字符串中提取列名称列表
    // fn extract_columns(sql: &str) -> Vec<String> {
    //     let dialect = GenericDialect {};
    //     let ast = Parser::parse_sql(&dialect, sql).unwrap();
    //     let expr_to_string = |expr: &Expr| -> String {
    //         match expr {
    //             Expr::Identifier(ident) => ident.value.clone(),
    //             Expr::CompoundIdentifier(idents) => idents
    //                 .iter()
    //                 .map(|i| i.value.clone())
    //                 .collect::<Vec<_>>()
    //                 .join("."),
    //             _ => format!("{:?}", expr), // 复杂表达式 fallback
    //         }
    //     };
    //     let mut columns = Vec::new();
    //     for stmt in ast {
    //         if let Statement::Query(query) = stmt {
    //             if let sqlparser::ast::SetExpr::Select(select) = *query.body {
    //                 for item in select.projection {
    //                     match item {
    //                         // SELECT col
    //                         SelectItem::UnnamedExpr(expr) => {
    //                             columns.push(expr_to_string(&expr));
    //                         }
    //
    //                         // SELECT col AS alias
    //                         SelectItem::ExprWithAlias { alias, .. } => {
    //                             columns.push(alias.value);
    //                         }
    //
    //                         // SELECT *
    //                         SelectItem::Wildcard(_) => {
    //                             columns.push("*".to_string());
    //                         }
    //
    //                         // SELECT table.*
    //                         SelectItem::QualifiedWildcard(obj_name, _) => {
    //                             columns.push(obj_name.to_string());
    //                         }
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     columns
    // }

    /// 生成映射函数的源代码字符串
    pub async fn make_mapper_source(&self, conn: &mut SqliteConnection, table: &TableInfo) -> String {
        //从 SQL 中提取参数名称列表
        let sql_params = Self::extract_params(self.sql.as_str());

        let mut sql = self.sql.clone();
        sql_params.iter().for_each(|it| {
            //将 SQL 中的 :param 替换为 ?
            sql = sql.replace(format!(":{}", it).as_str(), "?");
        });

        //当前表对应的结构体名称
        // let entity_name = table.get_entity_name();
        // let columns = Self::extract_columns(self.sql.as_str());
        // let mut struct_columns = Vec::new();
        // columns.iter().for_each(|it| {
        //     //如果 SQL 中有 * 号，并且函数返回值类型与实体类名称相同，则将实体类的所有列都加入到 struct_columns 中
        //     if it.ends_with("*") {
        //         if self.return_type == entity_name {
        //             //如果函数返回值类型与实体类名称相同，则将实体类的所有列都加入到 struct_columns 中
        //             table.columns.iter().for_each(|it| {
        //                 struct_columns.push(it.name.clone());
        //             });
        //             return;
        //         }
        //         //这里需要展开结构体
        //         return;
        //     }
        //     struct_columns.push(it.to_string());
        // });

        let mut return_struct_src = String::new();
        if self.return_type != table.get_entity_name(){//如果当前返回值不是表的Entity，只自动生产Entity
            let name_2_column: HashMap<String, &ColumnInfo> = table.columns.iter().map(|it|(it.name.clone(), it)).collect();

            //预编译sql，拿到matedata
            let member_src =   conn.prepare(sql.as_str()).await.unwrap().columns().iter().map(|it| {
                if let Some(exists_col) = name_2_column.get(it.name()){
                    return exists_col.make_member_src();
                }
                let is_nullable = match it.type_info().name() {
                    "INTEGER"=> false,
                    _ => true
                };
                ColumnInfo{
                    name: it.name().to_string(),
                    data_type: it.type_info().name().to_string(),
                    is_nullable,
                    ..Default::default()
                }.make_member_src()
            }).collect::<Vec<_>>().join("");
            return_struct_src = format!("#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]\npub struct {} {{ {} }}", self.return_type, member_src);
        }
        
        let sql_template = match self.crud_type {
            CrudType::Read => {
                if self.is_list {
                    r###"
                    [RETURN_STRUCT]
                    pub async fn [FUNC_NAME]<'e, E>(executor: E, [FUNC_PARAMS]) -> Result<Vec<[RETURN_TYPE]>, sqlx::Error>
                     where E: sqlx::Executor<'e, Database = sqlx::Sqlite>
                    {
                         sqlx::query_as!(
                             [RETURN_TYPE],
                             r#"[SQL]"#,
                             [SQL_PARAM]
                         )
                         .fetch_all(executor)
                         .await
                    }"###
                } else {
                    r###"
                    [RETURN_STRUCT]
                    pub async fn [FUNC_NAME]<'e, E>(executor: E, [FUNC_PARAMS]) -> Result<[RETURN_TYPE], sqlx::Error> 
                     where E: sqlx::Executor<'e, Database = sqlx::Sqlite>
                    {
                         sqlx::query_as!(
                             [RETURN_TYPE],
                             r#"[SQL]"#,
                             [SQL_PARAM]
                         )
                         .fetch_one(executor)
                         .await
                    }"###
                }
            }
            CrudType::Update | CrudType::Delete | CrudType::Create => {
                r##"
                    pub async fn [FUNC_NAME]<'e, E>(executor: E, [FUNC_PARAMS]) -> Result<u64, sqlx::Error> 
                     where E: sqlx::Executor<'e, Database = sqlx::Sqlite>
                    {
                        let rs = sqlx::query!(
                            r#"[SQL]"#,
                            [SQL_PARAM]
                        )
                            .execute(executor)
                            .await?;
                        let count = rs.rows_affected();
                        Ok(count)
                    }
                "##
            }
                _ => "",
        };

        sql_template
            .replace("[RETURN_STRUCT]", return_struct_src.as_str())
            .replace("[FUNC_NAME]", self.name.as_str())
            .replace("[FUNC_PARAMS]", self.params_str.as_str())
            .replace("[SQL_PARAM]", sql_params.join(", ").as_str())
            .replace("[RETURN_TYPE]", self.return_type.as_str())
            .replace("[SQL]", sql.as_str())
    }
}
