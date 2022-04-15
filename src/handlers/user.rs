use diesel::pg::Pg;
use diesel::Connection;

use crate::actix_web::web::{Json, Query};
use crate::context::UserInfo;
use crate::diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl, TextExpressionMethods};
use crate::error::Error;
use crate::handlers::DB;
use crate::response::List;
use crate::schema::*;
use crate::serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Serialize)]
pub struct User {
    id: i32,
    nickname: String,
}

#[derive(Debug, Deserialize)]
pub struct FindUserParams {
    phone: String,
    exclude_org_id: Option<i32>,
    page: i64,
    size: i64,
}

pub async fn find(Query(FindUserParams { phone, exclude_org_id, page, size }): Query<FindUserParams>, db: DB) -> Result<Json<List<User>>, Error> {
    let conn = db.get()?;
    let total: i64;
    let list: Vec<User>;
    if let Some(org_id) = exclude_org_id {
        total = users::table
            .left_join(users_organizations::table.left_join(organizations::table))
            .filter(users::phone.like(format!("%{phone}%")).and(organizations::id.ne(org_id).or(organizations::id.is_null())))
            .select(users::id)
            .distinct()
            .count()
            .get_result(&conn)?;
        list = users::table
            .left_join(users_organizations::table.left_join(organizations::table))
            .filter(users::phone.like(format!("%{phone}%")).and(organizations::id.ne(org_id).or(organizations::id.is_null())))
            .select((users::id, users::nickname))
            .distinct()
            .limit(size)
            .offset((page - 1) * size)
            .load(&conn)?;
    } else {
        total = users::table.filter(users::phone.like(format!("%{phone}%"))).count().get_result(&conn)?;
        list = users::table
            .filter(users::phone.like(format!("%{phone}%")))
            .select((users::id, users::nickname))
            .limit(size)
            .offset((page - 1) * size)
            .load(&db.get()?)?;
    }
    Ok(Json(List::new(list, total)))
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
    db: DB,
) -> Result<Json<List<User>>, Error> {
    let conn = db.get()?;
    let table = users::table.inner_join(users_organizations::table.inner_join(organizations::table));
    let mut count = table.clone().into_boxed();
    if let Some(phone) = &phone {
        count = count.filter(users::phone.like(format!("%{phone}%")));
    }
    if let Some(org_id) = org_id {
        count = count.filter(organizations::id.eq(org_id));
    }
    if let Some(exclude_org_id) = exclude_org_id {
        count = count.filter(organizations::id.ne(exclude_org_id));
    }
    let mut query = table.clone().select((users::id, users::nickname)).limit(size).offset((page - 1) * size).into_boxed::<Pg>();
    if let Some(phone) = &phone {
        query = query.filter(users::phone.like(format!("%{phone}%")));
    }
    if let Some(org_id) = org_id {
        query = query.filter(organizations::id.eq(org_id));
    }
    if let Some(exclude_org_id) = exclude_org_id {
        query = query.filter(organizations::id.ne(exclude_org_id));
    }
    let (users, total) = conn.transaction::<(Vec<User>, i64), Error, _>(|| {
        if phone.is_none() {
            let user_org_ids: Vec<i32> = users_organizations::table
                .filter(users_organizations::user_id.eq(me.id))
                .select(users_organizations::organization_id)
                .load(&conn)?;
            count = count.filter(organizations::id.eq_any(user_org_ids.clone()));
            query = query.filter(organizations::id.eq_any(user_org_ids.clone()));
        }
        let total = count.count().get_result(&conn)?;
        let users = query.load(&conn)?;
        Ok((users, total))
    })?;

    Ok(Json(List::new(users, total)))
}
