use crate::serde::Serialize;

#[derive(Debug, Serialize)]
pub struct List<T> {
    list: Vec<T>,
    total: i64,
}

impl<T> List<T> {
    pub fn new(list: Vec<T>, total: i64) -> Self {
        List { list, total }
    }
}

#[derive(Debug, Serialize)]
pub struct DeleteResponse {
    pub deleted: usize,
}

impl DeleteResponse {
    pub fn new(deleted: usize) -> Self {
        DeleteResponse { deleted }
    }
}

#[derive(Debug, Serialize)]
pub struct UpdateResponse {
    pub updated: usize,
}

impl UpdateResponse {
    pub fn new(updated: usize) -> Self {
        Self { updated }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateResponse {
    pub id: i32,
}
