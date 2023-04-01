use sqlx_insert::{table_name, Insertable};

#[derive(Debug, Insertable)]
#[table_name("vote_read_marks")]
pub struct VoteReadMarkInsertion {
    pub vote_id: i32,
    pub user_id: i32,
    pub version: i64,
}
