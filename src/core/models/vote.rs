use crate::core::models::question::QuestionCreate;
use chrono::NaiveDate;
use juju_macros::ToTuple;
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

#[derive(Debug, Serialize, ToTuple)]
pub struct Vote {
    pub id: i32,
    pub name: String,
    pub deadline: Option<NaiveDate>,
    pub organization_id: i32,
    pub version: i64,
    pub visibility: String,
    pub likes: i32,
    pub dislikes: i32,
    pub status: String,
    pub has_updated: bool,
    pub num_of_questions: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Insert {
    pub name: String,
    pub deadline: Option<NaiveDate>,
    pub organization_id: i32,
    pub visibility: String,
}

#[derive(Debug, Clone)]
pub struct Update {
    pub name: String,
    pub deadline: Option<NaiveDate>,
    pub version: i64,
    pub visibility: String,
}

#[derive(Debug)]
pub struct Query {
    pub uid: i32,
    pub organization_id_eq: Option<i32>,
}

pub struct ReadMarkInsert {
    pub vote_id: i32,
    pub user_id: i32,
    pub version: i64,
}
