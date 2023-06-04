use std::fmt::Debug;

use serde::Deserialize;

use crate::error::Error;
use crate::models::organization::{Insert as DBInsert, Organization as DBOrganization, OrganizationWithVoteInfo as DBOrganizationWithVoteInfo, Query as DBQuery, Update as DBUpdate};

use super::db::{OrganizationCommon, Storer, TxStorer};

#[derive(Debug, Deserialize)]
pub struct Create {
    pub name: String,
    pub description: String,
}

pub async fn create_organization<T>(mut tx: T, uid: i32, data: Create) -> Result<i32, Error>
where
    T: TxStorer,
{
    let exists = tx.exists(&data.name).await?;
    if exists {
        return Err(Error::BusinessError(format!("organization which has the same name already exists(name: {})", data.name)));
    }
    let id = OrganizationCommon::insert(
        &mut tx,
        DBInsert {
            name: data.name,
            version: 1,
            description: data.description,
        },
    )
    .await?;
    tx.add_member(id, uid).await?;
    tx.add_user_version(id, uid).await?;
    tx.add_manager(id, uid).await?;
    tx.commit().await?;
    Ok(id)
}

pub async fn joined_organizations<D>(db: &mut D, uid: i32, page: i64, size: i64) -> Result<(Vec<DBOrganizationWithVoteInfo>, i64), Error>
where
    D: Storer,
{
    let total = OrganizationCommon::count(
        db,
        DBQuery {
            member_id: Some(uid),
            ..Default::default()
        },
    )
    .await?;
    let orgs = OrganizationCommon::query(
        db,
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

pub async fn update_organization<T>(mut tx: T, uid: i32, id: i32, data: Update) -> Result<(), Error>
where
    T: TxStorer,
{
    let is_manager = tx.is_manager(id, uid).await?;
    if !is_manager {
        return Err(Error::BusinessError("no permission".into()));
    }
    let org = tx.get_for_update(id).await?;
    tx.update(
        id,
        DBUpdate {
            name: data.name,
            version: org.version + 1,
        },
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

pub async fn get_organization<D>(db: &mut D, id: i32) -> Result<DBOrganization, Error>
where
    D: Storer,
{
    let org = OrganizationCommon::get(db, id).await?;
    Ok(org)
}

pub async fn delete_organization<D>(db: &mut D, uid: i32, id: i32) -> Result<(), Error>
where
    D: Storer,
{
    if !OrganizationCommon::is_manager(db, id, uid).await? {
        return Err(Error::BusinessError("No permission".into()));
    }
    OrganizationCommon::delete(db, id).await
}

pub async fn add_manager<T>(tx: &mut T, id: i32, uid: i32) -> Result<(), Error>
where
    T: TxStorer,
{
    if OrganizationCommon::is_manager(tx, id, uid).await? {
        return Ok(());
    }
    OrganizationCommon::add_manager(tx, id, uid).await
}
