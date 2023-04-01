use sqlx::{query, query_as, query_scalar, FromRow, PgPool, QueryBuilder};

use crate::actix_web::web::{Data, Json, Path, Query};
use crate::context::UserInfo;
use crate::error::Error;
use crate::handlers::user::User;
use crate::models::{organization::Organization, vote::VoteWithStatuses};
use crate::request::Pagination;
use crate::response::{CreateResponse, DeleteResponse, UpdateResponse};
use crate::serde::{Deserialize, Serialize};

use crate::handlers::authorizer::Authorizer;
use crate::response::List;

pub async fn delete_organization(user_info: UserInfo, organization_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<DeleteResponse>, Error> {
    let organization_id = organization_id.into_inner().0;
    let (deleted,): (i32,) = query_as(
        r#"DELETE 
    FROM organizations  
    WHERE id IN (
        SELECT o.id 
        FROM users AS u 
        JOIN users_organizations AS uo ON u.id = uo.user_id 
        JOIN organizations AS o ON uo.organization_id = o.id
        WHERE u.id = $1 AND o.id = $2)"#,
    )
    .bind(user_info.id)
    .bind(organization_id)
    .fetch_one(&mut db.acquire().await?)
    .await?;
    Ok(Json(DeleteResponse::new(deleted)))
}

pub async fn detail(user_info: UserInfo, organization_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<Organization>, Error> {
    let organization_id = organization_id.into_inner().0;
    let mut tx = db.begin().await?;
    let org: Organization = query_as(
        r#"
        SELECT o.*
        FROM users AS u
        JOIN users_organizations AS uo ON u.id = uo.user_id
        JOIN organizations AS o ON uo.organization_id = o.id
        WHERE u.id = $1
        AND o.id = $2
        FOR SHARE"#,
    )
    .bind(user_info.id)
    .bind(organization_id)
    .fetch_one(&mut tx)
    .await?;
    query(
        "UPDATE organizations SET version = version + 1 WHERE id IN (
            SELECT o.id
            FROM users AS u
            JOIN users_organizations AS uo ON u.id = uo.user_id
            JOIN organizations AS o ON uo.organization_id = o.id
            WHERE u.id = $1
            AND o.id = $2)",
    )
    .bind(user_info.id)
    .bind(organization_id)
    .execute(&mut tx)
    .await?;
    tx.commit().await?;
    Ok(Json(org))
}

#[derive(Debug, Serialize, FromRow)]
pub struct Item {
    id: i32,
    name: String,
    version: i64,
    vote_count: i64,
    has_new_vote: bool,
}

pub async fn list(user_info: UserInfo, Query(Pagination { page, size }): Query<Pagination>, db: Data<PgPool>) -> Result<Json<List<Item>>, Error> {
    let mut tx = db.begin().await?;
    let (total,): (i64,) = query_as(
        "
        SELECT COUNT(*)
        FROM users AS u
        JOIN users_organizations AS uo ON u.id = uo.user_id
        JOIN organizations AS o ON uo.organization_id = o.id
        WHERE u.id = $1",
    )
    .bind(user_info.id)
    .fetch_one(&mut tx)
    .await?;
    let orgs = query_as(
        "
        SELECT 
            o.id, 
            o.name, 
            o.version, 
            COUNT(DISTINCT v.id) AS vote_count, 
            SUM(o.version) > SUM(orm.version) OR SUM(COALESCE(v.version, 0)) > SUM(COALESCE(vrm.version, 0)) OR SUM(COALESCE(q.version, 0)) > SUM(COALESCE(qrm.version, 0)) AS has_new_vote
        FROM users AS u
        JOIN users_organizations AS uo ON u.id = uo.user_id
        JOIN organizations AS o ON uo.organization_id = o.id
        JOIN organization_read_marks AS orm ON o.id = orm.organization_id
        LEFT JOIN votes AS v ON o.id = v.organization_id
        LEFT JOIN vote_read_marks AS vrm ON v.id = vrm.vote_id
        LEFT JOIN questions AS q ON v.id = q.vote_id
        LEFT JOIN question_read_marks AS qrm ON q.id = qrm.question_id
        WHERE u.id = $1
        AND orm.user_id = $1
        AND (vrm.user_id = $1 OR vrm.user_id IS NULL)
        AND (qrm.user_id = $1 OR qrm.user_id IS NULL)
        GROUP BY o.id, o.name, o.version
        LIMIT $2
        OFFSET $3",
    )
    .bind(user_info.id)
    .bind(size)
    .bind((page - 1) * size)
    .fetch_all(&mut tx)
    .await?;
    tx.commit().await?;
    Ok(Json(List::new(orgs, total)))
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrganizationCreation {
    name: String,
}

pub async fn create(user_info: UserInfo, Json(OrganizationCreation { name }): Json<OrganizationCreation>, db: Data<PgPool>) -> Result<Json<CreateResponse>, Error> {
    let mut tx = db.begin().await?;
    let (id,): (i32,) = query_as(
        "
        INSERT INTO organizations (name) values ($1) RETURNING id",
    )
    .bind(name)
    .fetch_one(&mut tx)
    .await?;
    query("INSERT INTO users_organizations (user_id, organization_id) VALUES ($1, $2)")
        .bind(user_info.id)
        .bind(id)
        .execute(&mut tx)
        .await?;
    query("INSERT INTO organization_read_marks (organization_id, user_id, version) VALUES ($1, $2, 1)")
        .bind(id)
        .bind(user_info.id)
        .execute(&mut tx)
        .await?;
    tx.commit().await?;
    Ok(Json(CreateResponse { id }))
}

#[derive(Deserialize)]
pub struct UpdateRequest {
    name: String,
}

pub async fn update(user_info: UserInfo, org_id: Path<(i32,)>, Json(req): Json<UpdateRequest>, db: Data<PgPool>) -> Result<Json<UpdateResponse>, Error> {
    let org_id = org_id.into_inner().0;
    let (updated,): (i32,) = query_as(
        "
    UPDATE organizations SET name = $1
    WHERE id IN (
        SELECT o.id
        FROM users AS u
        JOIN users_organizations AS uo ON u.id = uo.user_id
        JOIN organizations AS o ON uo.organization_id = o.id
        WHERE u.id = $1
        AND o.id = $3)",
    )
    .bind(req.name)
    .bind(user_info.id)
    .bind(org_id)
    .fetch_one(&mut db.acquire().await?)
    .await?;
    Ok(Json(UpdateResponse::new(updated as usize)))
}

pub async fn add_users(user_info: UserInfo, org_id: Path<(i32,)>, Json(user_ids): Json<Vec<i32>>, db: Data<PgPool>) -> Result<Json<()>, Error> {
    let org_id = org_id.into_inner().0;
    let mut tx = db.begin().await?;
    query(
        "
        SELECT * 
        FROM users_organizations AS uo
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
        INSERT INTO users_organizations (user_id, organization_id)",
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

// list all users which belongs to one organization
pub async fn users<T: Authorizer>(me: UserInfo, org_id: Path<(i32,)>, db: Data<PgPool>, authorizer: Data<T>) -> Result<Json<List<User>>, Error> {
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
    JOIN users_organizations AS uo ON u.id = uo.user_id
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
    JOIN users_organizations AS uo ON u.id = uo.user_id
    JOIN organizations AS o ON uo.organization_id = o.id
    WHERE o.id = $1",
    )
    .bind(org_id)
    .fetch_all(&mut conn)
    .await?;
    Ok(Json(List::new(list, total)))
}

pub async fn votes(user_info: UserInfo, param: Query<Pagination>, org_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<List<VoteWithStatuses>>, Error> {
    let org_id = org_id.into_inner().0;
    let mut tx = db.begin().await?;
    let total: i64 = query_scalar(
        "
    SELECT COUNT(*)
    FROM organizations AS o
    JOIN votes AS v ON o.id = v.organization_id
    WHERE o.id = $1",
    )
    .bind(org_id)
    .fetch_one(&mut tx)
    .await?;
    let votes: Vec<VoteWithStatuses> = query_as(
        "
    SELECT 
        v.*,
        CASE WHEN v.deadline <= NOW() THEN 'Expired' ELSE 'Active' END AS status,
        v.version > vrm.version AS has_updated
    FROM organizations AS o
    JOIN votes AS v ON o.id = v.organization_id
    JOIN vote_read_marks AS vrm ON v.id = vrm.vote_id
    WHERE vrm.id = $1
    AND o.id = $2
    LIMIT $3
    OFFSET $4",
    )
    .bind(user_info.id)
    .bind(org_id)
    .bind(param.size)
    .bind((param.page - 1) * param.size)
    .fetch_all(&mut tx)
    .await?;
    Ok(Json(List::new(votes, total)))
}
