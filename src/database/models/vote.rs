use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow)]
pub struct Vote {
    pub id: i32,
    pub name: String,
    pub deadline: Option<NaiveDate>,
    pub organization_id: i32,
    pub version: i64,
    pub visibility: String,
    pub status: String,
    pub has_updated: bool,
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
