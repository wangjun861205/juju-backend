use std::fmt::{Debug, Display};

use crate::error::Error;
use crate::models::organization::{Insert, Organization, Query, Update};

pub trait DatabaseManager {
    type Error: Display + Debug;

    async fn insert(&mut self, data: Insert) -> Result<i32, Self::Error>;
    async fn update(&mut self, id: i32, data: Update) -> Result<(), Self::Error>;
    async fn query(&mut self, param: Query, page: i64, size: i64) -> Result<Vec<Organization>, Self::Error>;
    async fn count(&mut self, param: Query) -> Result<i64, Self::Error>;
    async fn delete(&mut self, id: i32) -> Result<(), Self::Error>;
    async fn exists(&mut self, name: &str) -> Result<bool, Self::Error>;
    async fn add_member(&mut self, id: i32, uid: i32) -> Result<(), Self::Error>;
    async fn add_user_version(&mut self, id: i32, uid: i32) -> Result<(), Self::Error>;
    async fn update_user_version(&mut self, id: i32, uid: i32, version: i32) -> Result<(), Self::Error>;
    async fn add_manager(&mut self, id: i32, uid: i32) -> Result<(), Self::Error>;
}

pub async fn create_organization<M>(manager: &mut M, uid: i32, data: Insert) -> Result<i32, Error>
where
    M: DatabaseManager,
    Error: From<M::Error>,
{
    let exists = manager.exists(&data.name).await?;
    if exists {
        return Err(Error::BusinessError(format!("organization which has the same name already exists(name: {})", data.name)));
    }
    let id = manager.insert(data).await?;
    manager.add_member(id, uid).await?;
    manager.add_user_version(id, uid).await?;
    manager.add_manager(id, uid).await?;
    Ok(id)
}

pub async fn joined_organizations<M>(manager: &mut M, uid: i32, page: i64, size: i64) -> Result<(Vec<Organization>, i64), Error>
where
    M: DatabaseManager,
    Error: From<M::Error>,
{
    let total = manager
        .count(Query {
            member_id: Some(uid),
            ..Default::default()
        })
        .await?;
    let orgs = manager
        .query(
            Query {
                member_id: Some(uid),
                ..Default::default()
            },
            page,
            size,
        )
        .await?;
    Ok((orgs, total))
}
