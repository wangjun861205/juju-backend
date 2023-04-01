use serde::Deserialize;
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
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name("users")]
pub struct UserInsertion {
    pub nickname: String,
    pub email: String,
    pub phone: String,
    pub password: String,
    pub salt: String,
}
