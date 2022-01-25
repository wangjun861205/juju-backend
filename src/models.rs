use crate::schema::*;
use chrono::NaiveDate;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

use crate::diesel_derive_enum::DbEnum;

#[derive(Debug, Clone, Serialize, Deserialize, DbEnum)]
pub enum VoteStatus {
    Collecting,
    Closed,
}

#[derive(Debug, Clone, Serialize, Queryable, Identifiable)]
pub struct User {
    pub id: i32,
    pub nickname: String,
    pub phone: String,
    pub email: String,
    pub password: String,
    pub salt: String,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name = "users"]
pub struct UserInsertion {
    pub nickname: String,
    pub phone: String,
    pub email: String,
    pub password: String,
    pub salt: String,
}

#[derive(Debug, Clone, Serialize, Queryable, Identifiable)]
pub struct Organization {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name = "organizations"]
pub struct OrganizationInsertion {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Queryable, Associations, Identifiable)]
#[belongs_to(User)]
#[belongs_to(Organization)]
pub struct UsersOrganization {
    id: i32,
    user_id: i32,
    organization_id: i32,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name = "users_organizations"]
pub struct UsersOrganizationInsertion {
    pub user_id: i32,
    pub organization_id: i32,
}

#[derive(Debug, Clone, Serialize, Queryable, Identifiable, Associations)]
#[belongs_to(Organization)]
pub struct Vote {
    id: i32,
    name: String,
    deadline: Option<NaiveDate>,
    status: VoteStatus,
    organization_id: i32,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name = "votes"]
pub struct VoteInsertion {
    pub name: String,
    pub deadline: Option<NaiveDate>,
    pub status: VoteStatus,
    pub organization_id: i32,
}

#[derive(Debug, Clone, AsChangeset)]
#[table_name = "votes"]
pub struct VoteUpdation {
    pub name: String,
    pub deadline: Option<NaiveDate>,
    pub status: VoteStatus,
}

#[derive(Debug, Clone, Serialize, Queryable, Identifiable, Associations)]
#[belongs_to(Vote)]
pub struct Question {
    pub id: i32,
    pub description: String,
    pub vote_id: i32,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name = "questions"]
pub struct QuestionInsertion {
    pub description: String,
    pub vote_id: i32,
}

#[derive(Debug, Clone, Serialize, Identifiable, Queryable, Associations)]
#[belongs_to(Question)]
#[table_name = "options"]
pub struct Opt {
    id: i32,
    option: String,
    question_id: i32,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name = "options"]
pub struct OptInsertion {
    pub option: String,
    pub question_id: i32,
}

#[derive(Debug, Clone, Serialize, Identifiable, Queryable, Associations)]
#[belongs_to(User)]
#[belongs_to(Opt, foreign_key = "option_id")]
pub struct Answer {
    id: i32,
    user_id: i32,
    option_id: i32,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name = "answers"]
pub struct AnswerInsertion {
    pub user_id: i32,
    pub option_id: i32,
}

#[derive(Debug, Clone, Serialize, Identifiable, Queryable, Associations)]
#[belongs_to(User)]
#[belongs_to(Vote)]
pub struct Date {
    id: i32,
    d: NaiveDate,
    user_id: i32,
    vote_id: i32,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name = "dates"]
pub struct DateInsertion {
    pub d: NaiveDate,
    pub user_id: i32,
    pub vote_id: i32,
}

#[derive(Debug, Clone, Identifiable, Queryable)]
pub struct InviteCode {
    id: i32,
    code: String,
}
