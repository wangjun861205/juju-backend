use actix_web::web::Data;
use sqlx::{query_as, query_scalar, FromRow, PgPool, QueryBuilder};

use crate::actix_web::{
    web::{Json, Query},
    HttpResponse,
};
use crate::context::UserInfo;
use crate::core::models::ProfileUpdate;
use crate::core::user::{profile as profile_, search_by_phone, update_profile as update_profile_};
use crate::database::sqlx::PgSqlx;
use crate::error::Error;
use crate::models::user::Profile;
use crate::response::List;
use crate::serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, FromRow)]
pub struct User {
    id: i32,
    nickname: String,
}

#[derive(Debug, Deserialize)]
pub struct FindUserParams {
    phone: String,
    exclude_org_id: Option<i32>,
}

pub async fn find(Query(FindUserParams { phone, exclude_org_id }): Query<FindUserParams>, db: Data<PgPool>) -> Result<Json<Option<User>>, Error> {
    let store = PgSqlx::new(db.acquire().await?);
    let user = search_by_phone(store, phone, exclude_org_id).await?.map(|u| User { id: u.id, nickname: u.nickname });
    Ok(Json(user))
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    phone: Option<String>,
    org_id: Option<i32>,
    exclude_org_id: Option<i32>,
    page: i64,
    size: i64,
}

pub async fn list(
    me: UserInfo,
    Query(ListParams {
        phone,
        org_id,
        exclude_org_id,
        page,
        size,
    }): Query<ListParams>,
    db: Data<PgPool>,
) -> Result<Json<List<User>>, Error> {
    let mut conn = db.acquire().await?;
    let mut total_query = QueryBuilder::new(
        "
    SELECT COUNT(*)
    FROM users AS u
    JOIN organization_members AS uo ON u.id = uo.user_id
    JOIN organizations AS o ON uo.organization_id = o.id 
    WHERE 1 = 1 ",
    );
    if let Some(phone) = &phone {
        total_query.push("AND u.phone LIKE '%");
        total_query.push_bind(phone);
        total_query.push("%'");
    }
    if let Some(org_id) = org_id {
        total_query.push("AND o.id = ");
        total_query.push(org_id);
    }
    if let Some(exclude_org_id) = exclude_org_id {
        total_query.push("AND o.id <> ");
        total_query.push_bind(exclude_org_id);
    }
    let (total,): (i64,) = total_query.build_query_as().fetch_one(&mut conn).await?;
    let mut list_query = QueryBuilder::new(
        "SELECT u.id, u.nickname
    FROM users AS u
    JOIN organization_members AS uo ON u.id = uo.user_id
    JOIN organizations AS o ON uo.organization_id = o.id 
    WHERE 1 = 1",
    );
    if let Some(phone) = &phone {
        list_query.push(" AND u.phone LIKE '%");
        list_query.push_bind(phone);
        list_query.push("%'");
    }
    if let Some(org_id) = org_id {
        list_query.push(" AND o.id = ");
        list_query.push(org_id);
    }
    if let Some(exclude_org_id) = exclude_org_id {
        list_query.push(" AND o.id <> ");
        list_query.push_bind(exclude_org_id);
    }
    list_query.push(" LIMIT ");
    list_query.push_bind(size);
    list_query.push(" OFFSET ");
    list_query.push_bind((page - 1) * size);
    let users: Vec<User> = list_query.build_query_as().fetch_all(&mut conn).await?;
    Ok(Json(List::new(users, total)))
}

pub async fn profile(me: UserInfo, pool: Data<PgPool>) -> Result<Json<Profile>, Error> {
    let store = PgSqlx::new(pool.acquire().await?);
    let p = profile_(store, me.id).await?;
    Ok(Json(p))
}

pub async fn update_profile(me: UserInfo, Json(p): Json<Profile>, pool: Data<PgPool>) -> Result<HttpResponse, Error> {
    let store = PgSqlx::new(pool.acquire().await?);
    update_profile_(
        store,
        me.id,
        ProfileUpdate {
            nickname: p.nickname,
            avatar: p.avatar,
        },
    )
    .await?;
    Ok(HttpResponse::Ok().finish())
}
