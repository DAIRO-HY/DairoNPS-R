use sqlx::{Connection, SqliteConnection};

pub mod make_source;
pub mod read_table;
pub mod table_info;
pub mod mapper_info;
pub mod read_mapper;


pub async fn make(url: &str, mapper_dir: &str) {
    let mut conn = SqliteConnection::connect(url).await.unwrap();
    let mut tables = read_table::read(&mut conn).await;
    tables.iter_mut().for_each(|table| {
        table.mappers = read_mapper::read(&table.name, mapper_dir);
    });
    make_source::make(&mut conn, tables).await;
}