use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use sqlx::{query, query_as, query_scalar, PgPool, QueryBuilder};

use crate::actix_web::web::{Data, Json, Path, Query};
use crate::context::UserInfo;
use crate::core::models::vote::VoteQuery;
use crate::core::organization::delete_organization as delete_organization_core;
use crate::database::models::organization::OrganizationWithVoteInfo;
use crate::database::models::{organization::Organization, vote::Vote};
use crate::database::sqlx::PgSqlx;
use crate::error::Error;
use crate::handlers::user::User;
use crate::request::Pagination;
use crate::response::CreateResponse;
use crate::serde::{Deserialize, Serialize};

use crate::core::organization::{create_organization, get_organization, joined_organizations, update_organization, Create, Update};
use crate::handlers::authorizer::Authorizer;
use crate::response::List;

pub async fn delete_organization(user_info: UserInfo, organization_id: Path<(i32,)>, db: Data<PgPool>) -> Result<HttpResponse, Error> {
    let mut pg = PgSqlx::new(db.acquire().await?);
    delete_organization_core(&mut pg, user_info.id, organization_id.0).await?;
    Ok(HttpResponse::new(StatusCode::OK))
}

pub async fn detail(organization_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<Organization>, Error> {
    let conn = db.acquire().await?;
    let mut pg = PgSqlx::new(conn);
    let org = get_organization(&mut pg, organization_id.0).await?;
    Ok(Json(org))
}

pub async fn my_organizations(user_info: UserInfo, Query(Pagination { page, size }): Query<Pagination>, db: Data<PgPool>) -> Result<Json<List<OrganizationWithVoteInfo>>, Error> {
    let conn = db.acquire().await?;
    let mut pg = PgSqlx::new(conn);
    let (orgs, total) = joined_organizations(&mut pg, user_info.id, page, size).await?;
    Ok(Json(List::new(orgs, total)))
}

pub async fn create(user_info: UserInfo, Json(data): Json<Create>, db: Data<PgPool>) -> Result<Json<CreateResponse>, Error> {
    let tx = db.begin().await?;
    let pg = PgSqlx::new(tx);
    let id = create_organization(pg, user_info.id, data).await?;
    Ok(Json(CreateResponse { id }))
}

#[derive(Deserialize)]
pub struct UpdateRequest {
    name: String,
}

pub async fn update(user_info: UserInfo, org_id: Path<(i32,)>, Json(req): Json<UpdateRequest>, db: Data<PgPool>) -> Result<HttpResponse, Error> {
    let tx = db.begin().await?;
    let pg = PgSqlx::new(tx);
    update_organization(pg, user_info.id, org_id.0, Update { name: req.name }).await?;
    Ok(HttpResponse::new(StatusCode::OK))
}

pub async fn add_users(user_info: UserInfo, org_id: Path<(i32,)>, Json(user_ids): Json<Vec<i32>>, db: Data<PgPool>) -> Result<Json<()>, Error> {
    let org_id = org_id.into_inner().0;
    let mut tx = db.begin().await?;
    query(
        "
        SELECT * 
        FROM organization_members AS uo
        WHERE user_id = $1
        AND organization_id = $2
        FOR UPDATE",
    )
    .bind(user_info.id)
    .bind(org_id)
    .execute(&mut tx)
    .await?;
    QueryBuilder::new(
        "
        INSERT INTO organization_members (user_id, organization_id)",
    )
    .push_values(user_ids.iter(), |mut b, u| {
        b.push_bind(u);
        b.push_bind(org_id);
    })
    .build()
    .execute(&mut tx)
    .await?;
    for &uid in &user_ids {
        query(
            "
            INSERT INTO vote_read_marks (vote_id, user_id, version)
            SELECT (v.id, $1, 0)
            FROM organizations AS o
            JOIN votes AS v ON o.id = v.organization_id
            WHERE o.id = $2",
        )
        .bind(uid)
        .bind(org_id)
        .execute(&mut tx)
        .await?;
    }

    for &uid in &user_ids {
        query(
            "
            INSERT INTO question_read_marks (question_id, user_id, version)
            SELECT (q.id, $1, 0)
            FROM organizations as o
            JOIN votes AS v ON o.id = v.organization_id
            JOIN questions AS q ON v.id = q.vote_id
            WHERE o.id = $2",
        )
        .bind(uid)
        .bind(org_id)
        .execute(&mut tx)
        .await?;
    }
    Ok(Json(()))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddManager {
    user_id: i32,
    organization_id: i32,
}

pub async fn add_manager(Json(AddManager { user_id, organization_id }): Json<AddManager>, db: Data<PgPool>) -> Result<HttpResponse, Error> {
    let mut tx = db.begin().await?;
    if query_scalar("SELECT EXISTS(SELECT * FROM organization_managers WHERE user_id = $1 AND organization_id = $2)")
        .bind(user_id)
        .bind(organization_id)
        .fetch_one(&mut tx)
        .await?
    {
        return Ok(HttpResponse::Ok().finish());
    }
    query("INSERT INTO organization_managers (user_id, organization_id) VALUES ($1, $2)")
        .bind(user_id)
        .bind(organization_id)
        .execute(&mut tx)
        .await?;
    Ok(HttpResponse::Ok().finish())
}

// list all users which belongs to one organization
pub async fn members<T: Authorizer>(me: UserInfo, org_id: Path<(i32,)>, db: Data<PgPool>, authorizer: Data<T>) -> Result<Json<List<User>>, Error> {
    let org_id = org_id.into_inner().0;
    let mut conn = db.acquire().await?;
    let ok = authorizer.check_organization_read(me.id, org_id).await?;
    if !ok {
        return Err(Error::BusinessError("no permission".into()));
    }
    let (total,): (i64,) = query_as(
        "
    SELECT COUNT(*)
    FROM users AS u
    JOIN organization_members AS uo ON u.id = uo.user_id
    JOIN organizations AS o ON uo.organization_id = o.id
    WHERE o.id = $1",
    )
    .bind(org_id)
    .fetch_one(&mut conn)
    .await?;
    let list: Vec<User> = query_as(
        "
    SELECT u.id, u.nickname
    FROM users AS u
    JOIN organization_members AS uo ON u.id = uo.user_id
    JOIN organizations AS o ON uo.organization_id = o.id
    WHERE o.id = $1",
    )
    .bind(org_id)
    .fetch_all(&mut conn)
    .await?;
    Ok(Json(List::new(list, total)))
}

pub async fn votes(user_info: UserInfo, param: Query<Pagination>, org_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<List<Vote>>, Error> {
    let org_id = org_id.into_inner().0;
    let mut pg = PgSqlx::new(db.acquire().await?);
    let (votes, total) = crate::core::vote::query_votes(
        &mut pg,
        VoteQuery {
            organization_id: Some(org_id),
            uid: user_info.id,
            size: param.size,
            page: param.page,
        },
    )
    .await?;
    Ok(Json(List::new(votes, total)))
}

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub keyword: String,
    pub page: i32,
    pub size: i32,
}

pub async fn search(user_info: UserInfo, Query(SearchParams { keyword, page, size }): Query<SearchParams>, db: Data<PgPool>) -> Result<Json<List<Organization>>, Error> {
    let mut conn = db.acquire().await?;
    let total = query_scalar(
        "
    SELECT COUNT(o.id)
    FROM organizations AS o
    LEFT JOIN (SELECT organization_id FROM organization_members WHERE user_id = $1) AS uo ON o.id = uo.organization_id
    WHERE uo.organization_id IS NULL
    AND o.name LIKE $2",
    )
    .bind(user_info.id)
    .bind(format!("%{}%", &keyword))
    .fetch_one(&mut conn)
    .await?;
    let orgs = query_as(
        "
    SELECT o.*
    FROM organizations AS o
    LEFT JOIN (SELECT organization_id FROM organization_members WHERE user_id = $1) AS uo ON o.id = uo.organization_id
    WHERE uo.organization_id IS NULL
    AND o.name LIKE $2
    LIMIT $3
    OFFSET $4",
    )
    .bind(user_info.id)
    .bind(format!("%{}%", &keyword))
    .bind(size)
    .bind((page - 1) * size)
    .fetch_all(&mut conn)
    .await?;
    Ok(Json(List::new(orgs, total)))
}
