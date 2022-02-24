use crate::actix_web::{
    http::StatusCode,
    web::{Json, Path, Query},
    HttpResponse,
};
use crate::context::UserInfo;
use crate::diesel::{
    dsl::{any, sql, update as update_},
    sql_types::{BigInt, Bool},
    AsChangeset, BelongingToDsl, BoolExpressionMethods, Connection, ExpressionMethods, GroupByDsl, QueryDsl, RunQueryDsl,
};
use crate::error::Error;
use crate::handlers::DB;
use crate::models::{Organization, OrganizationInsertion, UsersOrganizationInsertion, Vote};
use crate::request::Pagination;
use crate::response::{DeleteResponse, UpdateResponse};
use crate::schema::{organizations, users, users_organizations, vote_update_marks, votes};
use crate::serde::{Deserialize, Serialize};

pub async fn delete_organization(user_info: UserInfo, Path((organization_id,)): Path<(i32,)>, db: DB) -> Result<Json<DeleteResponse>, Error> {
    let query = diesel::delete(organizations::table).filter(
        organizations::id.eq(any(users_organizations::table
            .filter(users_organizations::user_id.eq(user_info.id).and(users_organizations::organization_id.eq(organization_id)))
            .select(users_organizations::organization_id))),
    );
    Ok(Json(DeleteResponse::new(query.execute(&db.get()?)?)))
}

pub async fn organization_detail(user_info: UserInfo, Path((organization_id,)): Path<(i32,)>, db: DB) -> Result<Json<Organization>, Error> {
    let conn = db.get()?;
    let org: Organization = conn.transaction::<_, Error, _>(|| {
        let org = users::table
            .inner_join(users_organizations::table.inner_join(organizations::table))
            .select(organizations::all_columns)
            .filter(users::dsl::id.eq(user_info.id).and(organizations::dsl::id.eq(organization_id)))
            .for_share()
            .get_result::<Organization>(&db.get()?)?;
        Ok(org)
    })?;
    return Ok(Json(org));
}

#[derive(Debug, Serialize, Queryable)]
pub struct Item {
    id: i32,
    name: String,
    vote_count: i64,
    has_new_vote: bool,
}

pub async fn list(user_info: UserInfo, Query(Pagination { page, size }): Query<Pagination>, db: DB) -> Result<HttpResponse, Error> {
    use crate::response::List;
    use crate::schema::organizations::dsl as orgs_dsl;
    use crate::schema::users::dsl as users_dsl;
    use crate::schema::users_organizations::dsl as user_orgs_dsl;
    let conn = db.get()?;
    let (orgs, total) = conn.transaction::<(Vec<Item>, i64), Error, _>(|| {
        let total = users_dsl::users
            .inner_join(user_orgs_dsl::users_organizations.inner_join(orgs_dsl::organizations))
            .filter(users_dsl::id.eq(user_info.id))
            .count()
            .get_result(&conn)?;
        let orgs = users_dsl::users
            .inner_join(user_orgs_dsl::users_organizations.inner_join(orgs_dsl::organizations.left_join(votes::table.inner_join(vote_update_marks::table))))
            .filter(users_dsl::id.eq(user_info.id))
            .select((
                organizations::id,
                organizations::name,
                sql::<BigInt>("count(votes.id) as vote_count"),
                sql::<Bool>("BOOL_OR(vote_update_marks.has_updated) AS has_new_vote"),
            ))
            .group_by((organizations::id, organizations::name))
            .limit(size)
            .offset((page - 1) * size)
            .load::<Item>(&conn)?;
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

#[derive(Debug, Deserialize, AsChangeset)]
#[table_name = "organizations"]
pub struct UpdateRequest {
    name: String,
}

pub async fn update(user_info: UserInfo, Path((org_id,)): Path<(i32,)>, Json(req): Json<UpdateRequest>, db: DB) -> Result<Json<UpdateResponse>, Error> {
    let updated = update_(organizations::table)
        .filter(
            organizations::id
                .eq_any(
                    users::table
                        .inner_join(users_organizations::table.inner_join(organizations::table))
                        .filter(users::id.eq(user_info.id))
                        .select(organizations::id),
                )
                .and(organizations::id.eq(org_id)),
        )
        .set(req)
        .execute(&db.get()?)?;
    Ok(Json(UpdateResponse::new(updated)))
}
