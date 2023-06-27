use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[derive(Type, Serialize, Deserialize)]
pub enum ApplicationStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(FromRow, Serialize)]
pub struct JoinApplication {
    pub id: i32,
    pub user_id: i32,
    pub organization_id: i32,
    pub status: ApplicationStatus,
}

#[derive(Deserialize)]
pub struct JoinApplicationInsert {
    pub organization_id: i32,
}

