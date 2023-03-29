use crate::context::UserInfo;
use crate::error::Error;
use crate::{
    actix_web::web::{Data, Json, Path},
    response::{CreateResponse, DeleteResponse},
};

use crate::models::{Opt, Question, QuestionType};
use crate::response::List;
use crate::serde::{Deserialize, Serialize};
use crate::sqlx::{query, query_as, query_scalar, FromRow, PgPool};

#[derive(Debug, Deserialize)]
pub struct QuestionInsertion {
    description: String,
    type_: QuestionType,
}

#[derive(Debug, Deserialize)]
pub struct CreateRequest {
    pub question: QuestionInsertion,
    pub options: Vec<String>,
}

#[derive(Debug)]
pub struct UserID {
    id: i32,
}

pub async fn create(
    user_info: UserInfo,
    vote_id: Path<(i32,)>,
    Json(CreateRequest {
        question: QuestionInsertion { description, type_ },
        options,
    }): Json<CreateRequest>,
    db: Data<PgPool>,
) -> Result<Json<CreateResponse>, Error> {
    let vote_id = vote_id.into_inner().0;
    let mut tx = db.begin().await?;
    let (exists,): (bool,) = query_as(
        "
        SELECT EXISTS(
            SELECT *
            FROM users_organizations AS uo
            JOIN organizations AS o ON uo.organization_id = o.id
            JOIN votes AS v ON o.id = v.organization_id
            WHERE uo.user_id = $1
            AND v.id = $2)",
    )
    .bind(user_info.id)
    .bind(vote_id)
    .fetch_one(&mut tx)
    .await?;
    if !exists {
        return Err(Error::BusinessError("irrelative vote or vote not exists".into()));
    }
    let (id,): (i32,) = query_as("INSERT INTO questions (description, type_, vote_id, version) VALUES ($1, $2, $3, 1) RETURNING id")
        .bind(description)
        .bind(type_)
        .bind(vote_id)
        .fetch_one(&mut tx)
        .await?;
    for opt in options {
        query("INSERT INTO options (question_id, option) VALUES ($1, $2)").bind(id).bind(opt).execute(&mut tx).await?;
    }

    query("INSERT INTO question_read_marks (question_id, user_id, version) VALUES ($1, $2, 1)")
        .bind(id)
        .bind(user_info.id)
        .execute(&mut tx)
        .await?;
    tx.commit().await?;
    return Ok(Json(CreateResponse { id }));
}

#[derive(Debug, Serialize, FromRow)]
pub struct Item {
    id: i32,
    description: String,
    type_: QuestionType,
    version: i64,
    has_answered: bool,
    has_updated: bool,
}

pub async fn list(user_info: UserInfo, vote_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<List<Item>>, Error> {
    let vote_id = vote_id.into_inner().0;
    let mut conn = db.acquire().await?;
    let (total,): (i64,) = query_as(
        "
    SELECT COUNT(DISTINCT q.id)
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
    .fetch_one(&mut conn)
    .await?;
    let list = query_as(
        "
        SELECT q.id, q.description, q.type_, q.version, COUNT(distinct a.id) > 0 AS has_answered, q.version > SUM(qrm.version) AS has_updated
        FROM users AS u
        JOIN users_organizations AS uo ON u.id = uo.user_id
        JOIN organizations AS o ON uo.organization_id = o.id
        JOIN votes AS v ON o.id = v.organization_id
        JOIN questions AS q ON v.id = q.vote_id
        LEFT JOIN options AS op ON q.id = op.question_id
        LEFT JOIN answers AS a ON op.id = a.option_id AND u.id = a.user_id
        JOIN question_read_marks AS qrm ON q.id = qrm.question_id AND u.id = qrm.user_id
        WHERE u.id = $1
        AND v.id = $2
        GROUP BY q.id, q.description, q.type_, q.version",
    )
    .bind(user_info.id)
    .bind(vote_id)
    .fetch_all(&mut conn)
    .await?;
    Ok(Json(List::new(list, total)))
}

#[derive(Debug, Serialize)]
pub struct QuestionDetail {
    id: i32,
    description: String,
    type_: QuestionType,
    opts: Vec<Opt>,
}

pub async fn detail(user_info: UserInfo, qst_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<QuestionDetail>, Error> {
    let qst_id = qst_id.into_inner().0;
    let mut tx = db.begin().await?;
    let qst: Question = query_as(
        "SELECT q.*
    FROM users AS u
    JOIN users_organizations AS uo ON u.id = uo.user_id
    JOIN organizations AS o ON uo.organization_id = o.id
    JOIN votes AS v ON o.id = v.organization_id
    JOIN questions AS q ON v.id = q.vote_id
    WHERE u.id = $1
    AND q.id = $2
    FOR SHARE",
    )
    .bind(user_info.id)
    .bind(qst_id)
    .fetch_one(&mut tx)
    .await?;

    let opts: Vec<Opt> = query_as("SELECT * FROM options WHERE question_id = $1").bind(qst_id).fetch_all(&mut tx).await?;
    query(
        "
    UPDATE question_read_marks SET version = $1 WHERE user_id = $1 AND question_id = $2",
    )
    .bind(user_info.id)
    .bind(qst_id)
    .fetch_all(&mut tx)
    .await?;
    tx.commit().await?;
    Ok(Json(QuestionDetail {
        id: qst.id,
        description: qst.description,
        type_: qst.type_,
        opts: opts,
    }))
}

pub async fn delete(user_info: UserInfo, qst_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<DeleteResponse>, Error> {
    let qst_id = qst_id.into_inner().0;
    let (deleted,): (i32,) = query_as(
        "
    DELETE FROM questions
    WHERE id IN (
        SELECT q.id
        FROM users AS u
        JOIN users_organizations AS uo ON u.id = uo.user_id
        JOIN organizations AS o ON uo.organization_id = o.id
        JOIN votes AS v ON o.id = v.organization_id
        JOIN questions AS q ON v.id = q.vote_id
        WHERE u.id = $1
        AND q.id = $2)",
    )
    .bind(user_info.id)
    .bind(qst_id)
    .fetch_one(&mut db.acquire().await?)
    .await?;
    Ok(Json(DeleteResponse::new(deleted)))
}

#[derive(Debug, FromRow)]
struct QuestionWithOptions {
    question_id: i32,
    question_description: String,
    question_type: QuestionType,
    option_id: i32,
    option_option: String,
}

pub async fn questions_with_options_by_vote_id(user_info: UserInfo, vote_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<List<QuestionDetail>>, Error> {
    let vote_id = vote_id.into_inner().0;
    let total = query_scalar(
        "
            SELECT COUNT(*)
            FROM users AS u
            JOIN users_organizations AS uo ON u.id = uo.user_id
            JOIN votes AS v ON uo.organization_id = v.organization_id
            JOIN questions AS q ON v.id = q.vote_id
            WHERE u.id = $1 AND v.id = $2
    ",
    )
    .bind(user_info.id)
    .bind(vote_id)
    .fetch_one(&mut db.acquire().await?)
    .await?;
    let list: Vec<QuestionWithOptions> = query_as(
        "
    SELECT
        q.id AS question_id,
        q.description AS question_description,
        q.type_ AS question_type,
        o.id AS option_id,
        o.option AS option_option
    FROM 
        (SELECT
            q.id,
            q.description,
            q.type_
        FROM users AS u
        JOIN users_organizations AS uo ON u.id = uo.user_id
        JOIN votes AS v ON uo.organization_id = v.organization_id
        JOIN questions AS q ON v.id = q.vote_id
        WHERE u.id = $1 AND v.id = $2) AS q
        JOIN options AS o ON q.id = o.question_id
        ",
    )
    .bind(user_info.id)
    .bind(vote_id)
    .fetch_all(&mut db.acquire().await?)
    .await?;
    let list = list.into_iter().fold(Vec::<QuestionDetail>::new(), |mut l, q| {
        if let Some(mut last) = l.pop() {
            if last.id == q.question_id {
                last.opts.push(Opt {
                    id: q.option_id,
                    option: q.option_option,
                    question_id: q.question_id,
                });
                l.push(last);
                return l;
            }
            l.push(last);
        }
        l.push(QuestionDetail {
            id: q.question_id,
            description: q.question_description,
            type_: q.question_type,
            opts: vec![Opt {
                id: q.option_id,
                option: q.option_option,
                question_id: q.question_id,
            }],
        });
        return l;
    });
    Ok(Json(List::new(list, total)))
}
