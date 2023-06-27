#[derive(Debug)]
pub struct ProfileUpdate {
    pub nickname: String,
    pub avatar: Option<String>,
}

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx_insert::{table_name, Insertable};

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: i32,
    pub nickname: String,
    pub phone: String,
    pub email: String,
    pub password: String,
    pub salt: String,
    pub avatar: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name("users")]
pub struct UserInsertion {
    pub nickname: String,
    pub email: String,
    pub phone: String,
    pub password: String,
    pub salt: String,
    pub avatar: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub nickname: String,
    pub avatar: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Partial {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Option<String>>,
}

#[derive(Debug, Default)]
pub struct Patch {
    pub nickname: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub salt: Option<String>,
    pub avatar: Option<Option<String>>,
}
