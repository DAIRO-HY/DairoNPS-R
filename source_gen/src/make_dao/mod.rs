pub mod make_source;
pub mod read_table_from_sql;
pub mod table_info;

pub fn make(dir: &str) {
    let tables = read_table_from_sql::read(dir);
    make_source::make(tables);
}