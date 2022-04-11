use crate::actix_web::web::{Json, Query};
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
