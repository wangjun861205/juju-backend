use std::fmt::{Debug, Display};

use serde::Deserialize;

use crate::error::Error;
use crate::models::organization::{Insert as DBInsert, Organization as DBOrganization, OrganizationWithVoteInfo as DBOrganizationWithVoteInfo, Query as DBQuery, Update as DBUpdate};

pub trait DatabaseManager {
    type Error: Display + Debug;

    async fn insert(&mut self, data: DBInsert) -> Result<i32, Self::Error>;
    async fn update(&mut self, id: i32, data: DBUpdate) -> Result<(), Self::Error>;
    async fn query(&mut self, param: DBQuery, page: i64, size: i64) -> Result<Vec<DBOrganizationWithVoteInfo>, Self::Error>;
    async fn count(&mut self, param: DBQuery) -> Result<i64, Self::Error>;
    async fn delete(&mut self, id: i32) -> Result<(), Self::Error>;
    async fn exists(&mut self, name: &str) -> Result<bool, Self::Error>;
    async fn add_member(&mut self, id: i32, uid: i32) -> Result<(), Self::Error>;
    async fn add_user_version(&mut self, id: i32, uid: i32) -> Result<(), Self::Error>;
    async fn update_user_version(&mut self, id: i32, uid: i32, version: i32) -> Result<(), Self::Error>;
    async fn add_manager(&mut self, id: i32, uid: i32) -> Result<(), Self::Error>;
    async fn get(&mut self, id: i32) -> Result<DBOrganization, Self::Error>;
    async fn get_for_update(&mut self, id: i32) -> Result<DBOrganization, Self::Error>;
    async fn is_member(&mut self, id: i32, uid: i32) -> Result<bool, Self::Error>;
    async fn is_manager(&mut self, id: i32, uid: i32) -> Result<bool, Self::Error>;
}

#[derive(Debug, Deserialize)]
pub struct Create {
    pub name: String,
}

pub async fn create_organization<M>(manager: &mut M, uid: i32, data: Create) -> Result<i32, Error>
where
    M: DatabaseManager,
    Error: From<M::Error>,
{
    let exists = manager.exists(&data.name).await?;
    if exists {
        return Err(Error::BusinessError(format!("organization which has the same name already exists(name: {})", data.name)));
    }
    let id = manager.insert(DBInsert { name: data.name, version: 1 }).await?;
    manager.add_member(id, uid).await?;
    manager.add_user_version(id, uid).await?;
    manager.add_manager(id, uid).await?;
    Ok(id)
}

pub async fn joined_organizations<M>(manager: &mut M, uid: i32, page: i64, size: i64) -> Result<(Vec<DBOrganizationWithVoteInfo>, i64), Error>
where
    M: DatabaseManager,
    Error: From<M::Error>,
{
    let total = manager
        .count(DBQuery {
            member_id: Some(uid),
            ..Default::default()
        })
        .await?;
    let orgs = manager
        .query(
            DBQuery {
                member_id: Some(uid),
                ..Default::default()
            },
            page,
            size,
        )
        .await?;
    Ok((orgs, total))
}

pub struct Update {
    pub name: String,
}

pub async fn update_organization<M>(manager: &mut M, uid: i32, id: i32, data: Update) -> Result<(), Error>
where
    M: DatabaseManager,
    Error: From<M::Error>,
{
    let is_manager = manager.is_manager(id, uid).await?;
    if !is_manager {
        return Err(Error::BusinessError("no permission".into()));
    }
    let org = manager.get_for_update(id).await?;
    manager
        .update(
            id,
            DBUpdate {
                name: data.name,
                version: org.version + 1,
            },
        )
        .await?;
    Ok(())
}

pub async fn get_organization<M>(manager: &mut M, id: i32) -> Result<DBOrganization, Error>
where
    M: DatabaseManager,
    Error: From<M::Error>,
{
    let org = manager.get(id).await?;
    Ok(org)
}
