use crate::core::db::{Storer, UserCommon};
use crate::error::Error;
use crate::models::user::{Patch as UserPatch, Profile};

use super::db::OrganizationCommon;
use super::models::ProfileUpdate;

#[derive(Debug, Default)]
pub struct User {
    pub id: i32,
    pub nickname: String,
    pub phone: String,
    pub is_member: Option<bool>,
    pub is_manager: Option<bool>,
}
pub async fn search_by_phone<D>(mut db: D, phone: String, org_id: Option<i32>) -> Result<Option<User>, Error>
where
    D: Storer,
{
    if let Some(user) = UserCommon::get_by_phone(&mut db, phone).await? {
        let mut u = User {
            id: user.id,
            nickname: user.nickname,
            phone: user.phone,
            ..Default::default()
        };
        if let Some(org_id) = org_id {
            let is_member = OrganizationCommon::is_member(&mut db, org_id, user.id).await?;
            let is_manager = OrganizationCommon::is_manager(&mut db, org_id, user.id).await?;
            u.is_member = Some(is_member);
            u.is_manager = Some(is_manager);
        }
        return Ok(Some(u));
    }
    Ok(None)
}

pub async fn profile<D>(mut db: D, user_id: i32) -> Result<Profile, Error>
where
    D: Storer,
{
    let user = UserCommon::get(&mut db, user_id).await?;
    Ok(Profile {
        nickname: user.nickname,
        avatar: user.avatar,
    })
}

pub async fn update_profile<D>(mut db: D, user_id: i32, profile: ProfileUpdate) -> Result<(), Error>
where
    D: Storer,
{
    UserCommon::patch(
        &mut db,
        user_id,
        UserPatch {
            nickname: Some(profile.nickname),
            avatar: Some(profile.avatar),
            ..default::default()
        },
    )
    .await?;
    Ok(())
}
