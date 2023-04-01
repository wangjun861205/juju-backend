use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx_insert::{table_name, Insertable};

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Organization {
    pub id: i32,
    pub name: String,
    pub version: i64,
}

#[derive(Debug, Clone, Deserialize, Insertable)]
#[table_name("organizations")]
pub struct OrganizationInsertion {
    pub name: String,
}
