pub struct Pagination {
    limit: i64,
    offset: Option<i64>,
}

impl Pagination {
    pub fn new(limit: i64, offset: Option<i64>) -> Self {
        Self { limit, offset }
    }

    pub fn to_sql_clause(&self) -> String {
        let mut stmt = format!("LIMIT {} ", self.limit);
        if let Some(offset) = self.offset {
            stmt.push_str(&format!("OFFSET {}", offset));
        }
        stmt
    }
}
