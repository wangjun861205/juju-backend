use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx_insert::{table_name, Insertable};

#[derive(sqlx::Type)]
#[sqlx(type_name = "question_type")]
#[sqlx(rename_all = "UPPERCASE")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum QuestionType {
    #[default]
    Single,
    Multi,
}

#[derive(Debug, Clone, Serialize, FromRow, Default)]
pub struct QuestionWithStatuses {
    pub id: i32,
    pub description: String,
    pub vote_id: i32,
    pub type_: QuestionType,
    pub version: i64,
    pub has_answered: bool,
    pub has_updated: bool,
}

#[derive(Debug, Clone, Serialize, FromRow, Default)]
pub struct Question {
    pub id: i32,
    pub description: String,
    pub vote_id: i32,
    pub type_: QuestionType,
    pub version: i64,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name("questions")]
pub struct QuestionInsertion {
    pub description: String,
    pub vote_id: i32,
    pub type_: QuestionType,
    pub version: i64,
}

pub struct Query {
    pub vote_id: Option<i32>,
}

pub struct ReadMarkCreate {
    pub question_id: i32,
    pub user_id: i32,
    pub version: i64,
}
