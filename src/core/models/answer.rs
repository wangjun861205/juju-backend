use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Deserialize)]
pub struct Submit {
    pub question_id: i32,
    pub option_ids: Vec<i32>,
}

pub struct BulkSubmit {
    pub vote_id: i32,
    pub submissions: Vec<Submit>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Answer {
    id: i32,
    user_id: i32,
    option_id: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Insert {
    pub user_id: i32,
    pub option_id: i32,
}

pub struct Query {
    pub question_id_eq: Option<i32>,
    pub user_id_eq: Option<i32>,
}
