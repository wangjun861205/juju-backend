use sqlx::{query, query_as, FromRow};

use crate::actix_web::web::{Data, Json, Path};

use crate::chrono::NaiveDate;
use crate::context::UserInfo;
use crate::core::models::vote::VoteCreate;
use crate::core::models::{date::Date, question::Question, vote::Vote};
use crate::core::ports::repository::TxStore;
use crate::core::services::question::{question_detail, questions_with_in_vote};
use crate::core::services::vote::{create_vote, vote_detail};
use crate::database::sqlx::PgSqlx;
use crate::error::Error;
use crate::response::{CreateResponse, DeleteResponse, List, UpdateResponse};
use crate::serde::{Deserialize, Serialize};
use crate::sqlx::PgPool;

pub async fn create(user_info: UserInfo, Json(body): Json<VoteCreate>, db: Data<PgPool>) -> Result<Json<CreateResponse>, Error> {
    let tx = PgSqlx::new(db.begin().await?);
    let vote_id = create_vote(tx, user_info.id, body).await?;
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
    let tx = db.begin().await?;
    let mut store = PgSqlx::new(tx);
    let vote = vote_detail(&mut store, user_info.id, vote_id).await?;
    store.commit().await?;
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

async fn gen_question_report(user_info: UserInfo, question_id: i32, db: &Data<PgPool>) -> Result<QuestionReport, Error> {
    let mut conn = db.acquire().await?;
    let mut store = PgSqlx::new(db.acquire().await?);
    let question = question_detail(&mut store, user_info.id, question_id).await?;
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
        reports.push(gen_question_report(user_info.clone(), id, &db).await?)
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

pub async fn questions(user_info: UserInfo, vote_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<List<Question>>, Error> {
    let vote_id = vote_id.into_inner().0;
    let (list, total) = questions_with_in_vote(&mut PgSqlx::new(db.acquire().await?), user_info.id, vote_id).await?;
    Ok(Json(List::new(list, total)))
}
