use crate::core::models::option::{Opt, OptCreate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum QuestionType {
    #[default]
    Single,
    Multi,
}

#[derive(Debug, Deserialize)]
pub struct QuestionCreate {
    pub description: String,
    pub type_: QuestionType,
    pub options: Vec<OptCreate>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Question {
    pub id: i32,
    pub description: String,
    pub vote_id: i32,
    pub type_: String,
    pub version: i64,
    pub owner: i32,
    pub has_updated: bool,
    pub has_answered: bool,
    pub options: Vec<Opt>,
}

#[derive(Debug, Clone, Deserialize)]
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
    pub vote_id_eq: Option<i32>,
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
