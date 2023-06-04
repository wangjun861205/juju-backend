use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx_insert::{table_name, Insertable};

#[derive(Debug, sqlx::types::Type, Serialize, Deserialize, Clone)]
pub enum VoteVisibility {
    Public,
    Organization,
    WhiteList,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Vote {
    pub id: i32,
    pub name: String,
    pub deadline: Option<NaiveDate>,
    pub organization_id: i32,
    pub version: i64,
    pub visibility: VoteVisibility,
    pub status: String,
    pub has_updated: bool,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name("votes")]
pub struct VoteInsertion {
    pub name: String,
    pub deadline: Option<NaiveDate>,
    pub organization_id: i32,
    pub visibility: VoteVisibility,
}

#[derive(Debug, Clone)]
pub struct VoteUpdation {
    pub name: String,
    pub deadline: Option<NaiveDate>,
    pub version: i64,
    pub visibility: VoteVisibility,
}

pub struct ReadMarkCreate {
    pub vote_id: i32,
    pub user_id: i32,
    pub version: i64,
}
