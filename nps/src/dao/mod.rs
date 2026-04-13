pub mod channel_dao;
pub mod data_io_dao;
pub mod client_dao;
pub mod forward_dao;
pub mod system_config_dao;

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
