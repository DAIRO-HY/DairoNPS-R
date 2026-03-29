pub mod make_source;
pub mod read_table_from_sql;
pub mod table_info;
pub mod mapper_info;
pub mod read_mapper;


pub fn make(dir: &str, mapper_dir: &str) {
    let mut tables = read_table_from_sql::read(dir);
    tables.iter_mut().for_each(|table| {
        table.mappers = read_mapper::read(&table.name, mapper_dir);
    });
    make_source::make(tables);
}