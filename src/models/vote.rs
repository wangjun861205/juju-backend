use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx_insert::{table_name, Insertable};

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Vote {
    pub id: i32,
    pub name: String,
    pub deadline: Option<NaiveDate>,
    pub organization_id: i32,
    pub version: i64,
}

#[derive(Debug, Serialize, FromRow)]
pub struct VoteWithStatuses {
    pub id: i32,
    pub name: String,
    pub deadline: Option<NaiveDate>,
    pub organization_id: i32,
    pub version: i64,
    pub status: String,
    pub has_updated: bool,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name("votes")]
pub struct VoteInsertion {
    pub name: String,
    pub deadline: Option<NaiveDate>,
    pub organization_id: i32,
}

#[derive(Debug, Clone)]
pub struct VoteUpdation {
    pub name: String,
    pub deadline: Option<NaiveDate>,
    pub version: i64,
}
