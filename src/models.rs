use std::{ops::Bound, str::FromStr};

use crate::error::Error;
use crate::sqlx::FromRow;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::postgres::types::PgRange;
use sqlx_insert::{table_name, Insertable};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoteStatus {
    Collecting,
    Closed,
}

impl FromStr for VoteStatus {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Collecting" => Ok(Self::Collecting),
            "Closed" => Ok(Self::Closed),
            _ => Err(Error::BusinessError(format!("invalid vote status({})", s))),
        }
    }
}

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: i32,
    pub nickname: String,
    pub phone: String,
    pub email: String,
    pub password: String,
    pub salt: String,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name(users)]
pub struct UserInsertion {
    pub nickname: String,
    pub email: String,
    pub phone: String,
    pub password: String,
    pub salt: String,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Organization {
    pub id: i32,
    pub name: String,
    pub version: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrganizationInsertion {
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsersOrganization {
    id: i32,
    user_id: i32,
    organization_id: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsersOrganizationInsertion {
    pub user_id: i32,
    pub organization_id: i32,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Vote {
    pub id: i32,
    pub name: String,
    pub deadline: Option<NaiveDate>,
    pub organization_id: i32,
    pub version: i64,
}

#[derive(Debug, Clone, Deserialize)]
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

#[derive(sqlx::Type)]
#[sqlx(type_name = "question_type")]
#[sqlx(rename_all = "UPPERCASE")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuestionType {
    Single,
    Multi,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Question {
    pub id: i32,
    pub description: String,
    pub vote_id: i32,
    pub type_: QuestionType,
    pub version: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuestionInsertion {
    pub description: String,
    pub vote_id: i32,
    pub type_: QuestionType,
    pub version: i64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Opt {
    id: i32,
    option: String,
    question_id: i32,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name(options)]
pub struct OptInsertion {
    pub option: String,
    pub question_id: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Answer {
    id: i32,
    user_id: i32,
    option_id: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnswerInsertion {
    pub user_id: i32,
    pub option_id: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct DateRange {
    pub id: i32,
    pub range_: PgRange<NaiveDate>,
    pub vote_id: i32,
    pub user_id: i32,
}

#[derive(Debug, Clone)]
pub struct DateRangeInsertion {
    pub range_: (Bound<NaiveDate>, Bound<NaiveDate>),
    pub vote_id: i32,
    pub user_id: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Date {
    id: i32,
    date_: NaiveDate,
    user_id: i32,
    vote_id: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DateInsertion {
    pub date_: NaiveDate,
    pub user_id: i32,
    pub vote_id: i32,
}

#[derive(Debug, Clone)]
pub struct InviteCode {
    id: i32,
    code: String,
}

#[derive(Debug)]
pub struct VoteReadMarkInsertion {
    pub vote_id: i32,
    pub user_id: i32,
    pub version: i64,
}
