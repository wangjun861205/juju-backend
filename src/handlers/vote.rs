use sqlx::{query, query_as, FromRow, Postgres, QueryBuilder, Transaction};

use crate::actix_web::web::{Data, Json, Path};

use crate::chrono::NaiveDate;
use crate::context::UserInfo;
use crate::core::models::VoteCreate;
use crate::core::vote::create_vote;
use crate::database::sqlx::PgSqlx;
use crate::error::Error;
use crate::impls::uploaders::info_store::SqlxInfoStore;
use crate::impls::uploaders::local_storage::LocalStorage;
use crate::models::{
    date::Date,
    question::{Question, QuestionWithStatuses},
    vote::Vote,
};
use crate::response::{CreateResponse, DeleteResponse, List, UpdateResponse};
use crate::serde::{Deserialize, Serialize};
use crate::sqlx::PgPool;

pub async fn create(user_info: UserInfo, org_id: Path<(i32,)>, Json(body): Json<VoteCreate>, db: Data<PgPool>) -> Result<Json<CreateResponse>, Error> {
    let tx = PgSqlx::new(db.begin().await?);
    let uploader = LocalStorage::new("./uploads".into(), SqlxInfoStore::with_tx(db.begin().await?));
    let vote_id = create_vote(tx, uploader, body).await?;
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
        JOIN organization_members AS uo ON u.id = uo.user_id
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

pub async fn detail(user_info: UserInfo, vote_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<Vote>, Error> {
    let vote_id = vote_id.into_inner().0;
    let mut tx = db.begin().await?;
    let vote: Vote = query_as(
        "
    SELECT 
        v.*,
        CASE WHEN v.deadline < current_date THEN 'EXPIRED' ELSE 'COLLECTING' END AS status,
        v.version > vrm.version AS has_updated
    FROM votes AS v 
    JOIN vote_read_marks AS vrm ON v.id = vrm.vote_id
    WHERE vrm.user_id = $1
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
    tx.commit().await?;
    Ok(Json(vote))
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
    from organization_members as uo
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
    JOIN organization_members AS uo ON u.id = uo.user_id
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
        JOIN organization_members AS uo ON u.id = uo.user_id 
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

pub async fn questions(user_info: UserInfo, vote_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<List<QuestionWithStatuses>>, Error> {
    let vote_id = vote_id.into_inner().0;
    let mut conn = db.acquire().await?;
    let (total,): (i64,) = query_as(
        "
    SELECT COUNT(DISTINCT q.id)
    FROM votes AS v
    JOIN questions AS q ON v.id = q.vote_id
    WHERE v.id = $1",
    )
    .bind(vote_id)
    .fetch_one(&mut conn)
    .await?;
    let list = query_as(
        "
        SELECT 
            q.id, 
            q.description, 
            q.type_, 
            q.version, 
            q.vote_id,
            COUNT(distinct a.id) > 0 AS has_answered, 
            q.version > SUM(qrm.version) AS has_updated
        FROM votes AS v
        JOIN questions AS q ON v.id = q.vote_id
        JOIN question_read_marks AS qrm ON q.id = qrm.question_id AND qrm.user_id = $1
        LEFT JOIN options AS op ON q.id = op.question_id
        LEFT JOIN answers AS a ON op.id = a.option_id AND a.user_id = $1
        WHERE v.id = $2
        GROUP BY q.id, q.description, q.type_, q.version, q.vote_id",
    )
    .bind(user_info.id)
    .bind(vote_id)
    .fetch_all(&mut conn)
    .await?;
    Ok(Json(List::new(list, total)))
}

// SELECT
//     v.id,
//     v.name,
//     v.version > vrm.version,
//     bool_or(q.version > qrm.version)
// FROM votes AS v
// JOIN vote_read_marks AS vrm ON v.id = vrm.vote_id
// JOIN questions AS q ON v.id = q.vote_id
// JOIN question_read_marks AS qrm ON q.id = qrm.question_id
// GROUP BY v.id, v.name, v.version > vrm.version
