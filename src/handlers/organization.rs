use crate::actix_web::{
    http::StatusCode,
    web::{Json, Path, Query},
    HttpResponse,
};
use crate::context::UserInfo;
use crate::diesel::{dsl::any, BelongingToDsl, BoolExpressionMethods, Connection, ExpressionMethods, GroupByDsl, QueryDsl, RunQueryDsl};
use crate::error::Error;
use crate::handlers::DB;
use crate::models::{Organization, OrganizationInsertion, UsersOrganizationInsertion, Vote};
use crate::schema::{organizations, users, users_organizations, votes};
use crate::serde::{Deserialize, Serialize};

pub async fn delete_organization(user_info: UserInfo, Path((organization_id,)): Path<(i32,)>, db: DB) -> Result<HttpResponse, Error> {
    let query = diesel::delete(organizations::table).filter(
        organizations::id.eq(any(users_organizations::table
            .filter(users_organizations::user_id.eq(user_info.id).and(users_organizations::organization_id.eq(organization_id)))
            .select(users_organizations::organization_id))),
    );
    query.execute(&db.get()?)?;
    return Ok(HttpResponse::build(StatusCode::OK).finish());
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrgParam {
    page: i64,
    size: i64,
}

#[derive(Debug, Serialize, Queryable)]
pub struct Detail {
    id: i32,
    name: String,
    votes: Vec<Vote>,
}

pub async fn organization_detail(user_info: UserInfo, Path((organization_id,)): Path<(i32,)>, db: DB) -> Result<Json<Detail>, Error> {
    let org = users::table
        .inner_join(users_organizations::table.inner_join(organizations::table))
        .select(organizations::all_columns)
        .filter(users::dsl::id.eq(user_info.id).and(organizations::dsl::id.eq(organization_id)))
        .get_result::<Organization>(&db.get()?)?;
    let votes = Vote::belonging_to(&org).load::<Vote>(&db.get()?)?;
    return Ok(Json(Detail {
        id: org.id,
        name: org.name,
        votes: votes,
    }));
}

#[derive(Debug, Serialize, Queryable)]
pub struct OrganizationItem {
    id: i32,
    name: String,
    vote_count: i64,
}

pub async fn organization_list(user_info: UserInfo, page: Query<OrgParam>, db: DB) -> Result<HttpResponse, Error> {
    use crate::diesel::sql_types::BigInt;
    use crate::response::List;
    use crate::schema::organizations::dsl as orgs_dsl;
    use crate::schema::users::dsl as users_dsl;
    use crate::schema::users_organizations::dsl as user_orgs_dsl;
    let conn = db.get()?;
    let (orgs, total) = conn.transaction::<(Vec<OrganizationItem>, i64), Error, _>(|| {
        let total = users_dsl::users
            .inner_join(user_orgs_dsl::users_organizations.inner_join(orgs_dsl::organizations))
            .filter(users_dsl::id.eq(user_info.id))
            .count()
            .get_result(&conn)?;
        let orgs = users_dsl::users
            .inner_join(user_orgs_dsl::users_organizations.inner_join(orgs_dsl::organizations.left_join(votes::table)))
            .filter(users_dsl::id.eq(user_info.id))
            .select((organizations::id, organizations::name, crate::diesel::dsl::sql::<BigInt>("count(votes.id) as vote_count")))
            .group_by((organizations::id, organizations::name))
            .limit(page.0.size)
            .offset((page.0.page - 1) * page.0.size)
            .load::<OrganizationItem>(&conn)?;
        Ok((orgs, total))
    })?;
    return Ok(HttpResponse::build(StatusCode::OK).json(List::new(orgs, total)));
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrganizationCreation {
    name: String,
}

pub async fn create_organization(user_info: UserInfo, body: Json<OrganizationCreation>, db: DB) -> Result<HttpResponse, Error> {
    let conn = db.get()?;
    conn.transaction::<_, Error, _>(|| {
        let org_id = diesel::insert_into(organizations::table)
            .values(OrganizationInsertion { name: body.0.name })
            .returning(organizations::id)
            .get_result::<i32>(&conn)?;
        diesel::insert_into(users_organizations::table)
            .values(UsersOrganizationInsertion {
                user_id: user_info.id,
                organization_id: org_id,
            })
            .execute(&conn)?;
        Ok(())
    })?;
    return Ok(HttpResponse::build(StatusCode::OK).finish());
}
