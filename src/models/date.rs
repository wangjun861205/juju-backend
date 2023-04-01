use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::{postgres::types::PgRange, FromRow};
use sqlx_insert::{table_name, Insertable};
use std::ops::Bound;

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

#[derive(Debug, Clone, FromRow)]
pub struct DateRange {
    pub id: i32,
    pub range_: PgRange<NaiveDate>,
    pub vote_id: i32,
    pub user_id: i32,
}

#[derive(Debug, Clone, Insertable)]
#[table_name("date_ranges")]
pub struct DateRangeInsertion {
    pub range_: PgRange<NaiveDate>,
    pub vote_id: i32,
    pub user_id: i32,
}
