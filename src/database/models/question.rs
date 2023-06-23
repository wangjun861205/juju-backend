use crate::database::models::option::Create as OptCreate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx_insert::table_name;

#[derive(Debug, Clone, Serialize, FromRow, Default)]
pub struct QuestionWithStatuses {
    pub id: i32,
    pub description: String,
    pub vote_id: i32,
    pub type_: String,
    pub version: i64,
    pub has_answered: bool,
    pub has_updated: bool,
}

#[derive(Debug, Clone, Serialize, FromRow, Default)]
pub struct Question {
    pub id: i32,
    pub description: String,
    pub vote_id: i32,
    pub type_: String,
    pub version: i64,
}

#[derive(Debug, Clone, Deserialize)]
#[table_name("questions")]
pub struct Insert {
    pub description: String,
    pub vote_id: i32,
    pub type_: String,
    pub version: i64,
}

#[derive(Debug, Deserialize)]
pub struct Create {
    pub description: String,
    pub type_: String,
    pub version: i64,
    pub options: Vec<OptCreate>,
}

pub struct Query {
    pub vote_id: Option<i32>,
}

pub struct ReadMarkInsert {
    pub question_id: i32,
    pub user_id: i32,
    pub version: i64,
}

pub struct ReadMarkUpdate {
    pub question_id: i32,
    pub user_id: i32,
    pub version: i64,
}
