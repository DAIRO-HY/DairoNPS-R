#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Channel {
    pub id: i64,
    pub client_id: i64,
    pub name: String,
    pub mode: i64,
    pub server_port: i64,
    pub target_port: String,
    pub in_data: i64,
    pub out_data: i64,
    pub enable_state: i64,
    pub security_state: i64,
    pub acl_state: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub remark: Option<String>,
    pub error: Option<String>,
    pub version: i64,
}
/// 插入数据
pub async fn insert<'e, E>(executor: E, entity: Channel) -> Result<i64, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    sqlx::query_scalar!(
        "INSERT INTO channel (client_id, name, mode, server_port, target_port, in_data, out_data, enable_state, security_state, acl_state, created_at, updated_at, remark, error) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING id",
        entity.client_id, entity.name, entity.mode, entity.server_port, entity
        .target_port, entity.in_data, entity.out_data, entity.enable_state, entity
        .security_state, entity.acl_state, timestamp, timestamp, entity.remark, entity
        .error
    )
        .fetch_one(executor)
        .await
}

/// 通过主键查询一条数据
pub async fn select_one<'e, E>(executor: E, id:i64) -> Result<Channel, sqlx::Error>
where
    E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
{
    let row = sqlx::query_as!(
        Channel,
        "SELECT * FROM channel WHERE id = ?",
        id
    ).fetch_one(executor).await;
    row
}



// /// 通过主键查询一条数据
// pub async fn select_one<'e, E>(executor: E, id:i64) -> Channel
// where
//     E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
// {
//     let row = sqlx::query!(
//         "SELECT id, name FROM channel WHERE id = ?",
//         id
//     ).fetch_one(executor).await.unwrap();
//     row.

//     entity.unwrap()
// }
// pub fn select_all(
//     conn: &rusqlite::Connection,
// ) -> rusqlite::Result<Vec<Channel>, rusqlite::Error> {
//     const SQL: &str = "SELECT id, client_id, name, mode, server_port, target_port, in_data, out_data, enable_state, security_state, acl_state, created_at, updated_at, version FROM channel";
//     let mut stmt = conn.prepare(SQL)?;
//     stmt.query_map(
//             [],
//             |row| {
//                 Ok(Channel {
//                     id: row.get(0)?,
//                     client_id: row.get(1)?,
//                     name: row.get(2)?,
//                     mode: row.get(3)?,
//                     server_port: row.get(4)?,
//                     target_port: row.get(5)?,
//                     in_data: row.get(6)?,
//                     out_data: row.get(7)?,
//                     enable_state: row.get(8)?,
//                     security_state: row.get(9)?,
//                     acl_state: row.get(10)?,
//                     created_at: row.get(11)?,
//                     updated_at: row.get(12)?,
//                     version: row.get(13)?,
//                     ..Default::default()
//                 })
//             },
//         )?
//         .collect()
// }
// /// 更新数据
// pub fn update(conn: &rusqlite::Connection, entity: Channel) -> Option<rusqlite::Error> {
//     const SQL: &str = "UPDATE channel SET version = version + 1, client_id = ?, name = ?, mode = ?, server_port = ?, target_port = ?, in_data = ?, out_data = ?, enable_state = ?, security_state = ?, acl_state = ?, updated_at = ?, remark = ?, error = ? WHERE id = ? AND version = ?;";
//     match conn
//         .execute(
//             SQL,
//             rusqlite::params!(
//                 entity.client_id, entity.name, entity.mode, entity.server_port, entity
//                 .target_port, entity.in_data, entity.out_data, entity.enable_state,
//                 entity.security_state, entity.acl_state, std::time::SystemTime::now()
//                 .duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64,
//                 entity.remark, entity.error, entity.id, entity.version
//             ),
//         )
//     {
//         Ok(count) => {
//             if count == 0 {
//                 return Some(rusqlite::Error::QueryReturnedNoRows);
//             }
//         }
//         Err(e) => return Some(e),
//     }
//     None
// }
// /// 物理删除数据
// pub fn purge(conn: &rusqlite::Connection, id: i64) -> Option<rusqlite::Error> {
//     const SQL: &str = "DELETE FROM channel WHERE id = ?;";
//     match conn.execute(SQL, rusqlite::params!(id)) {
//         Ok(count) => {
//             if count == 0 {
//                 return Some(rusqlite::Error::QueryReturnedNoRows);
//             }
//         }
//         Err(e) => return Some(e),
//     }
//     None
// }
// pub fn select_active_by_client_id(
//     conn: &rusqlite::Connection,
//     client_id: i64,
// ) -> Result<Vec<Channel>, rusqlite::Error> {
//     const SQL: &str = r#"
// select channel.* from channel
//     left join client on channel.client_id = client.id
//     where channel.client_id = ? and client.enable_state = 1 and channel.enable_state = 1
// "#;
//     let mut stmt = conn.prepare(SQL)?;
//     stmt.query_map(
//             [client_id],
//             |row| {
//                 Ok(Channel {
//                     id: row.get(0)?,
//                     client_id: row.get(1)?,
//                     name: row.get(2)?,
//                     mode: row.get(3)?,
//                     server_port: row.get(4)?,
//                     target_port: row.get(5)?,
//                     in_data: row.get(6)?,
//                     out_data: row.get(7)?,
//                     enable_state: row.get(8)?,
//                     security_state: row.get(9)?,
//                     acl_state: row.get(10)?,
//                     created_at: row.get(11)?,
//                     updated_at: row.get(12)?,
//                     remark: row.get(13)?,
//                     error: row.get(14)?,
//                     version: row.get(15)?,
//                     ..Default::default()
//                 })
//             },
//         )?
//         .collect()
// }
