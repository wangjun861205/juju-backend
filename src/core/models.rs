use chrono::NaiveDate;

use crate::models::{question::QuestionType, vote::VoteVisibility};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OptionCreate {
    pub option: String,
    pub images: Vec<Vec<u8>>,
}

#[derive(Debug, Deserialize)]
pub struct QuestionCreate {
    pub description: String,
    pub type_: QuestionType,
    pub options: Vec<OptionCreate>,
}

#[derive(Debug, Deserialize)]
pub struct VoteCreate {
    pub name: String,
    pub deadline: Option<NaiveDate>,
    pub visibility: VoteVisibility,
    pub questions: Vec<QuestionCreate>,
    pub organization_id: i32,
}

#[derive(Debug, Default)]
pub struct VoteQuery {
    pub uid: i32,
    pub organization_id: Option<i32>,
    pub page: i64,
    pub size: i64,
}
