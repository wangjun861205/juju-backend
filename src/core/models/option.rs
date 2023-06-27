use serde::Deserialize;
use serde::Serialize;
use sqlx::FromRow;
use sqlx_insert::{table_name, Insertable};

#[derive(Debug, Deserialize)]
pub struct OptCreate {
    pub option: String,
    pub images: Vec<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Opt {
    pub id: i32,
    pub option: String,
    pub question_id: i32,
    pub images: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name("options")]
pub struct Insert {
    pub option: String,
    pub images: Vec<String>,
    pub question_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct Create {
    pub option: String,
    pub images: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Query {
    pub ids: Option<Vec<i32>>,
    pub question_id: Option<i32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
