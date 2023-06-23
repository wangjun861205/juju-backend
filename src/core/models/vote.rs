use crate::core::models::question::QuestionCreate;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum VoteVisibility {
    Public,
    Organization,
    WhiteList,
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
