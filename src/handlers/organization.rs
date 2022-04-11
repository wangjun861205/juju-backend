use crate::context::UserInfo;
use crate::diesel::{
    dsl::{any, exists, insert_into, select, sql, update as update_},
    helper_types::Eq,
    sql_types::{BigInt, Bool},
    AsChangeset, BoolExpressionMethods, Connection, ExpressionMethods, GroupByDsl, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl,
};
use crate::error::Error;
use crate::handlers::user::User;
use crate::handlers::DB;
use crate::models::{Organization, OrganizationInsertion, UsersOrganizationInsertion};
use crate::request::Pagination;
use crate::response::{CreateResponse, DeleteResponse, UpdateResponse};
use crate::schema::{organizations, question_read_marks, questions, users, users_organizations, vote_read_marks, votes};
use crate::serde::{Deserialize, Serialize};
use crate::{
    actix_web::web::{Data, HttpResponse, Json, Path, Query},
    schema::organization_read_marks,
};

use crate::authorizer::PgAuthorizer;
use crate::handlers::authorizer::Authorizer;
use crate::response::List;

pub async fn delete_organization(user_info: UserInfo, Path((organization_id,)): Path<(i32,)>, db: DB) -> Result<Json<DeleteResponse>, Error> {
    let query = diesel::delete(organizations::table).filter(
        organizations::id.eq(any(users_organizations::table
            .filter(users_organizations::user_id.eq(user_info.id).and(users_organizations::organization_id.eq(organization_id)))
            .select(users_organizations::organization_id))),
    );
    Ok(Json(DeleteResponse::new(query.execute(&db.get()?)?)))
}

pub async fn detail(user_info: UserInfo, Path((organization_id,)): Path<(i32,)>, db: DB) -> Result<Json<Organization>, Error> {
    let conn = db.get()?;
    let org: Organization = conn.transaction::<_, Error, _>(|| {
        let org = users::table
            .inner_join(users_organizations::table.inner_join(organizations::table))
            .select(organizations::all_columns)
            .filter(users::dsl::id.eq(user_info.id).and(organizations::dsl::id.eq(organization_id)))
            .for_share()
            .get_result::<Organization>(&db.get()?)?;
        update_(organization_read_marks::table)
            .filter(organization_read_marks::user_id.eq(user_info.id).and(organization_read_marks::organization_id.eq(organization_id)))
            .set(organization_read_marks::version.eq(org.version))
            .execute(&conn)?;
        Ok(org)
    })?;
    return Ok(Json(org));
}

#[derive(Debug, Serialize, Queryable)]
pub struct Item {
    id: i32,
    name: String,
    version: i64,
    vote_count: i64,
    has_new_vote: bool,
}

pub async fn list(user_info: UserInfo, Query(Pagination { page, size }): Query<Pagination>, db: DB) -> Result<Json<List<Item>>, Error> {
    let conn = db.get()?;
    let (orgs, total) = conn.transaction::<(Vec<Item>, i64), Error, _>(|| {
        let total = users::table
            .inner_join(users_organizations::table.inner_join(organizations::table))
            .filter(users::id.eq(user_info.id))
            .count()
            .get_result(&conn)?;
        let orgs = users::table
            .inner_join(
                users_organizations::table.inner_join(
                    organizations::table
                        .inner_join(organization_read_marks::table)
                        .left_join(votes::table.left_join(vote_read_marks::table).left_join(questions::table.left_join(question_read_marks::table))),
                ),
            )
            .select((
                organizations::id,
                organizations::name,
                organizations::version,
                sql::<BigInt>("COUNT(DISTINCT votes.id) as vote_count"),
                sql::<Bool>("SUM(organizations.version) > SUM(organization_read_marks.version) OR SUM(votes.version) > SUM(vote_read_marks.version) OR SUM(questions.version) > SUM(question_read_marks.version) AS has_new_vote"),
            ))
            .filter(users::id.eq(user_info.id).and(organization_read_marks::user_id.eq(user_info.id)).and(vote_read_marks::user_id.eq(user_info.id).or(vote_read_marks::user_id.is_null())).and(question_read_marks::user_id.eq(user_info.id).or(question_read_marks::user_id.is_null())))
            .group_by((organizations::id, organizations::name, organizations::version))
            .limit(size)
            .offset((page - 1) * size)
            .load::<Item>(&conn)?;
        Ok((orgs, total))
    })?;
    return Ok(Json(List::new(orgs, total)));
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrganizationCreation {
    name: String,
}

pub async fn create(user_info: UserInfo, body: Json<OrganizationCreation>, db: DB) -> Result<Json<CreateResponse>, Error> {
    let conn = db.get()?;
    let id = conn.transaction::<_, Error, _>(|| {
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
        diesel::insert_into(organization_read_marks::table)
            .values((
                organization_read_marks::organization_id.eq(org_id),
                organization_read_marks::user_id.eq(user_info.id),
                organization_read_marks::version.eq(1),
            ))
            .execute(&conn)?;
        Ok(org_id)
    })?;
    return Ok(Json(CreateResponse { id: id }));
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

pub async fn add_users(user_info: UserInfo, Path((org_id,)): Path<(i32,)>, Json(user_ids): Json<Vec<i32>>, db: DB) -> Result<HttpResponse, Error> {
    let conn = db.get()?;
    conn.transaction::<_, Error, _>(|| {
        users_organizations::table
            .filter(users_organizations::user_id.eq(user_info.id).and(users_organizations::organization_id.eq(org_id)))
            .select(users_organizations::id)
            .for_update()
            .first::<i32>(&conn)?;
        insert_into(users_organizations::table)
            .values(
                user_ids
                    .iter()
                    .map(|&v| (users_organizations::user_id.eq(v), users_organizations::organization_id.eq(org_id)))
                    .collect::<Vec<(_, _)>>(),
            )
            .execute(&conn)?;
        let vote_ids: Vec<i32> = organizations::table
            .inner_join(votes::table)
            .select(votes::id)
            .filter(organizations::id.eq(org_id))
            .for_update()
            .load(&conn)?;
        insert_into(vote_read_marks::table)
            .values(
                vote_ids
                    .iter()
                    .map(|&vid| {
                        user_ids
                            .iter()
                            .map(|&uid| (vote_read_marks::vote_id.eq(vid), vote_read_marks::user_id.eq(uid), vote_read_marks::version.eq(0)))
                            .collect::<Vec<(Eq<_, i32>, Eq<_, i32>, Eq<_, i64>)>>()
                    })
                    .flatten()
                    .collect::<Vec<_>>(),
            )
            .execute(&conn)?;
        let question_ids: Vec<i32> = votes::table
            .inner_join(questions::table)
            .select(questions::id)
            .filter(votes::id.eq_any(vote_ids))
            .for_update()
            .load(&conn)?;
        insert_into(question_read_marks::table)
            .values(
                question_ids
                    .iter()
                    .map(|&qid| {
                        user_ids
                            .iter()
                            .map(|&uid| (question_read_marks::question_id.eq(qid), question_read_marks::user_id.eq(uid), question_read_marks::version.eq(0)))
                            .collect::<Vec<(Eq<_, i32>, Eq<_, i32>, Eq<_, i64>)>>()
                    })
                    .flatten()
                    .collect::<Vec<(_, _, _)>>(),
            )
            .execute(&conn)?;
        Ok(())
    })?;
    Ok(HttpResponse::Ok().finish())
}

// list all users which belongs to one organization
pub async fn users<T: Authorizer>(me: UserInfo, Path((org_id,)): Path<(i32,)>, db: DB, authorizer: Data<T>) -> Result<Json<List<User>>, Error> {
    let conn = db.get()?;
    let ok = authorizer.check_organization_read(me.id, org_id)?;
    if !ok {
        return Err(Error::BusinessError("no permission".into()));
    }
    let total: i64 = users::table
        .inner_join(users_organizations::table.inner_join(organizations::table))
        .filter(organizations::id.eq(org_id))
        .count()
        .get_result(&conn)?;
    let list: Vec<User> = users::table
        .inner_join(users_organizations::table.inner_join(organizations::table))
        .select((users::id, users::nickname))
        .filter(organizations::id.eq(org_id))
        .load(&conn)?;
    Ok(Json(List::new(list, total)))
}
