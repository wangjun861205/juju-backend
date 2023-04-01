use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx_insert::{table_name, Insertable};

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Answer {
    id: i32,
    user_id: i32,
    option_id: i32,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name("answers")]
pub struct AnswerInsertion {
    pub user_id: i32,
    pub option_id: i32,
}
