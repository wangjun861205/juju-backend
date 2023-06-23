use crate::models::{question::QuestionType, vote::VoteVisibility};
use chrono::NaiveDate;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct QuestionCreate {
    pub description: String,
    pub type_: QuestionType,
    pub options: Vec<OptCreate>,
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

pub struct UploadedFile<I> {
    pub id: I,
    pub name: String,
    pub extension: String,
    pub content: Vec<u8>,
    pub owner_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct UploadedFileCreate {
    pub name: String,
    pub extension: String,
    pub content: Vec<u8>,
    pub owner_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct OptCreate {
    pub option: String,
    pub images: Vec<String>,
}

#[derive(Debug)]
pub struct ProfileUpdate {
    pub nickname: String,
    pub avatar: Option<String>,
}
