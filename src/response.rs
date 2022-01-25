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
