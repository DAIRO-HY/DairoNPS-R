
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct MapperInfo {
    pub sql: String,
    pub func: String,
}