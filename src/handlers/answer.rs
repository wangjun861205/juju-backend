use serde::Deserialize;
use sqlx::query_scalar;

use crate::actix_web::{
    http::StatusCode,
    web::{Data, Json, Path},
    HttpResponse,
};
use crate::context::UserInfo;
use crate::core::models::question::QuestionType;
use crate::error::Error;
use crate::serde::Serialize;
use crate::sqlx::{query, query_as, FromRow, PgPool, QueryBuilder};

pub async fn submit_answer(user_info: UserInfo, qst_id: Path<(i32,)>, Json(answer): Json<Vec<i32>>, db: Data<PgPool>) -> Result<HttpResponse, Error> {
    let mut tx = db.begin().await?;
    let qst_id = qst_id.into_inner().0;
    let is_answer_valid: bool = query_scalar(
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
        "DELETE FROM answers WHERE id IN (
            SELECT a.id 
            FROM questions AS q
            JOIN options AS o ON q.id = o.question_id
            JOIN answers AS a ON o.id = a.option_id
            WHERE q.id = $1)",
    )
    .bind(qst_id)
    .execute(&mut tx)
    .await?;
    QueryBuilder::new("INSERT INTO answers (user_id, option_id)")
        .push_values(answer.into_iter(), |mut b, a| {
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
    question_type: String,
    options: Vec<OptionItem>,
    answers: Vec<i32>,
}

pub async fn answer_list(user_info: UserInfo, qst_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<AnswerList>, Error> {
    let mut tx = db.begin().await?;
    let qst_id = qst_id.into_inner().0;
    let question_type: String = query_scalar(
        r#"
    SELECT q.type_
    FROM users AS u
    JOIN organization_members AS uo ON u.id = uo.user_id
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
        question_type,
        options,
        answers: answers.into_iter().map(|v| v.0).collect(),
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitAnswer {
    question_id: i32,
    option_ids: Vec<i32>,
}

pub async fn submit_answers(db: Data<PgPool>, user_info: UserInfo, vote_id: Path<(i32,)>, Json(answers): Json<Vec<SubmitAnswer>>) -> Result<HttpResponse, Error> {
    let vote_id = vote_id.into_inner().0;
    let mut tx = db.begin().await?;
    let is_valid: bool = query_scalar(
        "
    SELECT EXISTS(
        SELECT *
        FROM users AS u
        JOIN organization_members AS uo ON u.id = uo.user_id
        JOIN votes AS v ON uo.organization_id = v.id
        WHERE u.id = $1 AND v.id = $2
        FOR SHARE
    )",
    )
    .bind(user_info.id)
    .bind(vote_id)
    .fetch_one(&mut tx)
    .await?;
    if !is_valid {
        tx.rollback().await?;
        return Err(Error::ActixError(actix_web::error::ErrorForbidden("no permission")));
    }
    query(
        "DELETE FROM answers WHERE user_id = $1 AND question_id IN (
        SELECT id FROM questions WHERE vote_id = $1)",
    )
    .bind(user_info.id)
    .bind(vote_id)
    .execute(&mut tx)
    .await?;
    QueryBuilder::new("INSERT INTO answers (user_id, option_id, question_id)")
        .push_values(answers.iter().flat_map(|a| a.option_ids.iter().map(|&id| (a.question_id, id))), |mut s, a| {
            s.push_bind(user_info.id);
            s.push_bind(a.1);
            s.push_bind(a.0);
        })
        .build()
        .execute(&mut tx)
        .await?;
    tx.commit().await?;
    Ok(HttpResponse::build(StatusCode::CREATED).finish())
}
