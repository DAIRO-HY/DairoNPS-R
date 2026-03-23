pub mod channel_dao;
pub mod channel_data_dao;
pub mod client_dao;

/// 生成的DAO文件，包含了增删改查等基本操作的函数定义，以及分页查询条件实体类的定义。
pub enum QueryModel<T>{
    None,
    Equal(T),
    Like(T),
    GreaterThan(T),
    LessThan(T),
    GreaterThanOrEqual(T),
    LessThanOrEqual(T),
    In(Vec<T>),
    NotEqual(T),
    NotLike(T),
    NotIn(Vec<T>),
    Between(T, T),
    NotBetween(T, T),
}

// 如果希望对所有 T 都能 Default，则需要 T: Default 约束
impl<T: Default> Default for QueryModel<T> {
    fn default() -> Self {
        Self::None
    }
}
#[derive(Default)]
struct Test {
    pub id: QueryModel<i32>,
    pub name: QueryModel<String>,
}

// 宏生成dao结构体和构造函数
#[macro_export]
macro_rules! make_string_struct {
    ($name:ident) => {
        pub struct $name {
            connection: Option<rusqlite::Connection>,
        }

        impl $name {
            pub fn share() -> Self {
                Self { connection: None }
            }

            pub fn from(conn: rusqlite::Connection) -> Self {
                Self {
                    connection: Some(conn),
                }
            }

            // pub async fn conn(&self) -> &rusqlite::Connection {
            //     if let Some(conn) = &self.connection {
            //         &conn
            //     } else {
            //         let conn = crate::util::db_util::connection().await;
            //         &conn
            //     }
            // }
        }
    };
}

// 宏生成dao结构体和构造函数
make_string_struct!(ChannelDataDao);
