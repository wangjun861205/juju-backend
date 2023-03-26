use serde::Deserialize;
use sqlx::query_scalar;

use crate::actix_web::{
    http::StatusCode,
    web::{Data, Json, Path},
    HttpResponse,
};
use crate::context::UserInfo;
use crate::error::Error;
use crate::models::QuestionType;
use crate::serde::Serialize;
use crate::sqlx::{query, query_as, FromRow, PgPool, QueryBuilder};

pub async fn submit_answer(user_info: UserInfo, qst_id: Path<(i32,)>, Json(answer): Json<Vec<i32>>, db: Data<PgPool>) -> Result<HttpResponse, Error> {
    let mut tx = db.begin().await?;
    let qst_id = qst_id.into_inner().0;
    let (is_vote_valid,): (bool,) = query_as(
        r#"
    WITH now AS (SELECT now())
    SELECT EXISTS(
        SELECT *
        FROM options AS op
        JOIN questions AS q ON op.question_id = q.id
        JOIN votes AS v ON q.vote_id = v.id
        JOIN organizations AS og ON v.organization_id = og.id
        JOIN users_organizations AS uo ON o.id = uo.organization_id
        WHERE q.id = $1
        AND (v.deadline IS NULL OR v.deatline > now)
        AND u.id = $2
    )"#,
    )
    .bind(qst_id)
    .bind(user_info.id)
    .fetch_one(&mut tx)
    .await?;
    if !is_vote_valid {
        return Err(Error::BusinessError("vote not exists or invalid vote status".into()));
    }
    let (is_answer_valid,): (bool,) = query_as(
        r#"select (ids @> $1) as is_valid from (
                select q.id, array_agg(o.id) as ids
                from questions as q join options as o on q.id = o.question_id
                where q.id = $2 
                group by q.id
            ) as t"#,
    )
    .bind(answer.clone())
    .bind(qst_id)
    .fetch_one(&mut tx)
    .await?;
    if !is_answer_valid {
        return Err(Error::BusinessError("invalid answer".into()));
    }
    query(
        r#"
    DELETE answers WHERE id IN (
        SELECT a.id 
        FROM questions AS q
        JOIN options AS o ON q.id = options.question_id
        JOIN answers AS a ON o.id = a.option_id
        WHERE q.id = $1)"#,
    )
    .bind(qst_id)
    .execute(&mut tx)
    .await?;
    QueryBuilder::new(r#"INSERT INTO answers (user_id, option_id)"#)
        .push_tuples(answer.into_iter(), |mut b, a| {
            b.push_bind(user_info.id);
            b.push_bind(a);
        })
        .build()
        .execute(&mut tx)
        .await?;
    tx.commit().await?;
    Ok(HttpResponse::build(StatusCode::OK).finish())
}

#[derive(Debug, Serialize, FromRow)]
pub struct OptionItem {
    id: i32,
    option: String,
}

#[derive(Debug, Serialize)]
pub struct AnswerList {
    question_type: QuestionType,
    options: Vec<OptionItem>,
    answers: Vec<i32>,
}

pub async fn answer_list(user_info: UserInfo, qst_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<AnswerList>, Error> {
    let mut tx = db.begin().await?;
    let qst_id = qst_id.into_inner().0;
    let (question_type,): (QuestionType,) = query_as(
        r#"
    SELECT q.type
    FROM users AS u
    JOIN users_organizations AS uo ON u.id = uo.user_id
    JOIN organizations AS o ON uo.organization_id = o.id
    JOIN votes AS v ON o.id = v.organization_id
    JOIN questions AS q ON v.id = q.vote_id
    WHERE u.id = $1 AND q.id = $2"#,
    )
    .bind(user_info.id)
    .bind(qst_id)
    .fetch_one(&mut tx)
    .await?;
    let options: Vec<OptionItem> = query_as(
        r#"
    SELECT o.id AS id, o.option AS option
    FROM questions AS q
    JOIN options AS o ON q.id = o.question_id
    WHERE q.id = $1"#,
    )
    .bind(qst_id)
    .fetch_all(&mut tx)
    .await?;

    let answers: Vec<(i32,)> = query_as(
        r#"
        SELECT o.id
        FROM questions AS q
        JOIN  options AS o ON q.id = o.question_id
        JOIN answers AS a ON o.id = a.option_id
        WHERE a.user_id = $1 AND q.id = $2"#,
    )
    .bind(user_info.id)
    .bind(qst_id)
    .fetch_all(&mut tx)
    .await?;
    tx.commit().await?;
    Ok(Json(AnswerList {
        question_type: question_type,
        options: options,
        answers: answers.into_iter().map(|v| v.0).collect(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct SubmitAnswer {
    question_id: i32,
    option_id: i32,
}

pub async fn submit_answers(db: Data<PgPool>, user_info: UserInfo, vote_id: Path<(i32,)>, Json(answers): Json<Vec<SubmitAnswer>>) -> Result<usize, Error> {
    let mut tx = db.begin().await?;
    let isValid: bool = query_scalar(
        "
    SELECT EXISTS(
        SELECT *
        FROM users AS u
        JOIN users_organizations AS uo ON u.id = uo.user_id
        JOIN votes AS v ON uo.organization_id = v.id
        WHERE u.id = $1 AND v.id = $2
        FOR SHARE
    )",
    )
    .bind(user_info.id)
    .bind(vote_id.0)
    .fetch_one(&mut tx)
    .await?;
    Ok(0)
}
