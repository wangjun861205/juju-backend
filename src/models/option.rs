use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx_insert::{table_name, Insertable};

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Opt {
    pub id: i32,
    pub option: String,
    pub question_id: i32,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name("options")]
pub struct OptInsert {
    pub option: String,
    pub images: Vec<Vec<u8>>,
    pub question_id: i32,
}
