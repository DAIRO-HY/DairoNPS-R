pub mod make_source;
pub mod read_table;
pub mod table_info;
pub mod mapper_info;
pub mod read_mapper;


pub async fn make(url: &str, mapper_dir: &str) {
    let mut tables = read_table::read(url).await;
    tables.iter_mut().for_each(|table| {
        table.mappers = read_mapper::read(&table.name, mapper_dir);
    });
    make_source::make(tables);
}