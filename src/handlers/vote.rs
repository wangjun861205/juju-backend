use sqlx::{query, query_as, FromRow, QueryBuilder};

use crate::actix_web::web::{Data, Json, Path, Query};

use crate::chrono::{DateTime, NaiveDate, Utc};
use crate::context::UserInfo;
use crate::error::Error;
use crate::models::{Date, Question, Vote, VoteStatus};
use crate::request::Pagination;
use crate::response::{CreateResponse, DeleteResponse, List, UpdateResponse};
use crate::serde::{Deserialize, Serialize};
use crate::sqlx::PgPool;

#[derive(Debug, Clone, Deserialize)]
pub struct Creation {
    name: String,
    deadline: Option<DateTime<Utc>>,
}

pub async fn create(user_info: UserInfo, org_id: Path<(i32,)>, Json(body): Json<Creation>, db: Data<PgPool>) -> Result<Json<CreateResponse>, Error> {
    let org_id = org_id.into_inner().0;
    let mut tx = db.begin().await?;
    let (exists,): (bool,) = query_as(
        "
    SELECT EXISTS(
        SELECT *
        FROM users AS u
        JOIN users_organizations AS uo ON u.id = uo.user_id
        JOIN organizations AS o ON uo.organization_id = o.id
        WHERE u.id = $1
        AND o.id = $2)",
    )
    .bind(user_info.id)
    .bind(org_id)
    .fetch_one(&mut tx)
    .await?;
    if !exists {
        return Err(Error::BusinessError("irrelative organization".into()));
    }

    let (vote_id,): (i32,) = query_as("INSERT INTO votes (name, deadline, organization_id) VALUES ($1, $2, $3) RETURNING id")
        .bind(body.name)
        .bind(body.deadline.map(|dl| dl.naive_utc().date()))
        .bind(org_id)
        .fetch_one(&mut tx)
        .await?;
    let user_ids: Vec<i32> = query_as(
        "
    SELECT u.id
    FROM users AS u
    JOIN users_organizations AS uo ON u.id = uo.user_id
    JOIN organizations AS o ON uo.organization_id = o.id
    WHERE o.id = $1",
    )
    .bind(org_id)
    .fetch_all(&mut tx)
    .await?
    .into_iter()
    .map(|v: (i32,)| v.0)
    .collect();

    QueryBuilder::new("INSERT INTO vote_read_marks (user_id, vote_id, version) ")
        .push_values(user_ids.into_iter(), |mut b, v| {
            b.push_bind(v);
            b.push_bind(vote_id);
            b.push_bind(0);
        })
        .build()
        .execute(&mut tx)
        .await?;
    tx.commit().await?;
    Ok(Json(CreateResponse { id: vote_id }))
}

#[derive(Debug, Clone, Deserialize)]
pub struct VoteUpdation {
    name: String,
    deadline: Option<NaiveDate>,
}

pub async fn update(user_info: UserInfo, vote_id: Path<(i32,)>, Json(VoteUpdation { name, deadline }): Json<VoteUpdation>, db: Data<PgPool>) -> Result<Json<UpdateResponse>, Error> {
    let vote_id = vote_id.into_inner().0;
    let mut conn = db.acquire().await?;
    let (updated,): (i64,) = query_as(
        "WITH updated AS (UPDATE votes
    SET name = $1, deadline = $2, version = version + 1
    WHERE id = $3
    AND id IN (
        SELECT v.id
        FROM users AS u
        JOIN users_organizations AS uo ON u.id = uo.user_id
        JOIN organizations AS o ON uo.organization_id = o.id
        JOIN votes AS v ON o.id = v.organization_id
        WHERE u.id = $4)
    RETURNING *)
    SELECT COUNT(*) FROM updated",
    )
    .bind(name)
    .bind(deadline)
    .bind(vote_id)
    .bind(user_info.id)
    .fetch_one(&mut conn)
    .await?;
    Ok(Json(UpdateResponse { updated: updated as usize }))
}

#[derive(Debug, Serialize, FromRow)]
pub struct Item {
    id: i32,
    name: String,
    deadline: Option<NaiveDate>,
    version: i64,
    status: String,
    has_updated: bool,
}

#[derive(Debug, Serialize)]
struct VoteDetail {
    vote: Vote,
    dates: Vec<Date>,
    questions: Vec<Question>,
}

pub async fn list(user_info: UserInfo, param: Query<Pagination>, org_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<List<Item>>, Error> {
    let org_id = org_id.into_inner().0;
    let mut tx = db.begin().await?;
    let (total,): (i64,) = query_as(
        "
    SELECT COUNT(*)
    FROM users AS u
    JOIN users_organizations AS uo ON u.id = uo.user_id
    JOIN organizations AS o ON uo.organization_id = o.id
    JOIN votes AS v ON o.id = v.organization_id
    WHERE u.id = $1
    AND o.id = $2",
    )
    .bind(user_info.id)
    .bind(org_id)
    .fetch_one(&mut tx)
    .await?;
    let votes: Vec<Item> = query_as(
        "
    SELECT 
        v.id, 
        v.name, 
        v.deadline, 
        v.version,
        CASE WHEN v.deadline <= NOW() THEN 'Active' ELSE 'Expired' END AS status,
        SUM(v.version) > SUM(vrm.version) OR SUM(COALESCE(q.version, 0)) > SUM(COALESCE(qrm.version, 0)) AS has_updated
    FROM users AS u
    JOIN users_organizations AS uo ON u.id = uo.user_id
    JOIN organizations AS o ON uo.organization_id = o.id
    JOIN votes AS v ON o.id = v.organization_id
    JOIN vote_read_marks AS vrm ON v.id = vrm.vote_id AND u.id = vrm.user_id
    LEFT JOIN questions AS q ON v.id = q.vote_id
    LEFT JOIN question_read_marks AS qrm ON q.id = qrm.question_id AND u.id = qrm.user_id
    WHERE u.id = $1
    AND o.id = $2
    GROUP BY v.id, v.name, v.deadline, v.version, status
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

#[derive(Debug, Serialize)]
pub struct Detail {
    id: i32,
    name: String,
    deadline: Option<NaiveDate>,
    status: VoteStatus,
}

pub async fn detail(user_info: UserInfo, vote_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<Detail>, Error> {
    let vote_id = vote_id.into_inner().0;
    let mut tx = db.begin().await?;
    let vote: Vote = query_as(
        "
    SELECT v.*
    FROM users AS u
    JOIN users_organizations AS uo ON u.id = uo.user_id
    JOIN organizations AS o ON uo.organization_id = o.id
    JOIN votes AS v ON o.id = v.organization_id
    WHERE u.id = $1
    AND v.id = $2
    FOR SHARE",
    )
    .bind(user_info.id)
    .bind(vote_id)
    .fetch_one(&mut tx)
    .await?;
    query(
        "
    UPDATE vote_read_marks SET version = $1 WHERE user_id = $2 AND vote_id = $3",
    )
    .bind(vote.version)
    .bind(user_info.id)
    .bind(vote_id)
    .execute(&mut tx)
    .await?;
    Ok(Json(Detail {
        id: vote.id,
        name: vote.name,
        deadline: vote.deadline.clone(),
        status: if let Some(dl) = &vote.deadline {
            if dl < &Utc::now().date_naive() {
                VoteStatus::Closed
            } else {
                VoteStatus::Collecting
            }
        } else {
            VoteStatus::Collecting
        },
    }))
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct OptionReport {
    option: String,
    percentage: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuestionReport {
    question: String,
    options: Vec<OptionReport>,
}

async fn gen_question_report(question_id: i32, db: &Data<PgPool>) -> Result<QuestionReport, Error> {
    let mut conn = db.acquire().await?;
    let question: Question = query_as("SELECT * FROM questions WHERE id = $1").bind(question_id).fetch_one(&mut conn).await?;
    let opts = query_as(
        r#"
    select o.option as option, (count(distinct a.id)::float / (count(distinct uo.user_id))::float * 10000)::int as percentage
    from users_organizations as uo
    join organizations as org on uo.organization_id = org.id
    join votes as v on org.id = v.organization_id
    join questions as q on v.id = q.vote_id
    join options as o on q.id = o.question_id
    left join answers as a on o.id = a.option_id
    where q.id = $1
    group by option"#,
    )
    .bind(question_id)
    .fetch_all(&mut conn)
    .await?;
    Ok(QuestionReport {
        question: question.description,
        options: opts,
    })
}

pub async fn question_reports(user_info: UserInfo, vote_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<Vec<QuestionReport>>, Error> {
    let vote_id = vote_id.into_inner().0;
    let qids: Vec<(i32,)> = query_as(
        "
    SELECT q.id
    FROM users AS u
    JOIN users_organizations AS uo ON u.id = uo.user_id
    JOIN organizations AS o ON uo.organization_id = o.id
    JOIN votes AS v ON o.id = v.organization_id
    JOIN questions AS q ON v.id = q.vote_id
    WHERE u.id = $1
    AND v.id = $2",
    )
    .bind(user_info.id)
    .bind(vote_id)
    .fetch_all(&mut db.acquire().await?)
    .await?;
    let mut reports = Vec::new();
    for (id,) in qids {
        reports.push(gen_question_report(id, &db).await?)
    }

    Ok(Json(reports))
}

pub async fn delete_vote(user_info: UserInfo, vote_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<DeleteResponse>, Error> {
    let vote_id = vote_id.into_inner().0;
    let (deleted,): (i32,) = query_as(
        "
    DELETE 
    FROM votes 
    WHERE id IN (
        SELECT v.id 
        FROM users AS u 
        JOIN users_organizations AS uo ON u.id = uo.user_id 
        JOIN organizations AS o ON uo.organization_id = o.id 
        JOIN votes AS v ON o.id = v.organization_id 
        WHERE u.id = $1 AND v.id = $2)",
    )
    .bind(user_info.id)
    .bind(vote_id)
    .fetch_one(&mut db.acquire().await?)
    .await?;
    Ok(Json(DeleteResponse::new(deleted)))
}

pub async fn question_ids(vote_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<Vec<i32>>, Error> {
    let vid = vote_id.into_inner().0;
    let ids: Vec<(i32,)> = query_as("SELECT id FROM questions WHERE vote_id = $1").bind(vid).fetch_all(&mut db.acquire().await?).await?;
    Ok(Json(ids.into_iter().map(|v| v.0).collect()))
}
