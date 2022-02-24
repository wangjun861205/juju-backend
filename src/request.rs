use crate::serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub page: i64,
    pub size: i64,
}
