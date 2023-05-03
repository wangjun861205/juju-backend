use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx_insert::{table_name, Insertable};

#[derive(Debug, Clone, Serialize, FromRow, Default)]
pub struct Organization {
    pub id: i32,
    pub name: String,
    pub version: i64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, FromRow, Default)]
pub struct OrganizationWithVoteInfo {
    pub id: i32,
    pub name: String,
    pub version: i64,
    pub vote_count: Option<i64>,
    pub has_new_vote: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name("organizations")]
pub struct Insert {
    pub name: String,
    pub version: i64,
    pub description: String,
}

#[derive(Debug)]
pub struct Update {
    pub name: String,
    pub version: i64,
}

#[derive(Default)]
pub struct Query {
    pub name_eq: Option<String>,
    pub name_like: Option<String>,
    pub member_id: Option<i32>,
}
