use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use sqlx::{query, query_as, query_scalar, QueryBuilder};

use crate::context::UserInfo;
use crate::error::Error;
use crate::serde::Deserialize;
use crate::serde::Serialize;
use crate::sqlx::{FromRow, PgPool};
use crate::{
    actix_web::web::{Data, Json, Path},
    models::question::QuestionType,
};

#[derive(Debug, Serialize, FromRow)]
pub struct Item {
    id: i32,
    option: String,
    checked: bool,
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    question_type: QuestionType,
    items: Vec<Item>,
}

// pub async fn list(user_info: UserInfo, qst_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<ListResponse>, Error> {
//     let qst_id = qst_id.into_inner().0;
//     let mut conn = db.acquire().await?;
//     let (question_type,): (QuestionType,) = query_as(
//         r#"
//         SELECT q.type_
//         FROM users AS u
//         JOIN organization_members AS uo ON u.id = uo.user_id
//         JOIN organizations AS o ON uo.organization_id = o.id
//         JOIN votes AS v ON o.id = v.organization_id
//         JOIN questions AS q ON v.id = q.vote_id
//         WHERE u.id = $1 AND q.id = $2"#,
//     )
//     .bind(user_info.id)
//     .bind(qst_id)
//     .fetch_one(&mut conn)
//     .await?;
//     let items: Vec<Item> = query_as(
//         r#"
//         SELECT op.id, op.option, a.id IS NOT NULL AS checked
//         FROM users AS u
//         JOIN organization_members AS uo on u.id = uo.user_id
//         JOIN organizations AS og ON uo.organization_id = og.id
//         JOIN votes AS v ON og.id = v.organization_id
//         JOIN questions AS q ON v.id = q.vote_id
//         JOIN options AS op ON q.id = op.question_id
//         LEFT JOIN answers AS a ON op.id = a.option_id AND u.id = a.user_id
//         WHERE u.id = $1 AND q.id = $2"#,
//     )
//     .bind(user_info.id)
//     .bind(qst_id)
//     .fetch_all(&mut conn)
//     .await?;
//     Ok(Json(ListResponse { question_type, items }))
// }

#[derive(Debug, Clone, Deserialize)]
pub struct OptAdd {
    pub option: String,
}

pub async fn add_opts(user_info: UserInfo, qst_id: Path<(i32,)>, Json(options): Json<Vec<String>>, db: Data<PgPool>) -> Result<HttpResponse, Error> {
    let qst_id = qst_id.into_inner().0;
    let mut tx = db.begin().await?;
    let (org_id, vote_id): (i32, i32) = query_as(
        "
        SELECT o.id, v.id
        FROM users AS u
        JOIN organization_members AS uo ON u.id = uo.user_id
        JOIN organizations AS o ON uo.organization_id = o.id
        JOIN votes AS v ON o.id = v.organization_id
        JOIN questions AS q ON v.id = q.vote_id
        WHERE u.id = $1 AND q.id = $2
        FOR UPDATE",
    )
    .bind(user_info.id)
    .bind(qst_id)
    .fetch_one(&mut tx)
    .await?;

    QueryBuilder::new("INSERT INTO options (question_id, option)")
        .push_values(options.into_iter(), |mut b, o| {
            b.push_bind(qst_id);
            b.push_bind(o);
        })
        .push("RETURNING id")
        .build()
        .execute(&mut tx)
        .await?;

    query(
        "UPDATE organizations 
        SET version = version + 1
        WHERE id = $1",
    )
    .bind(org_id)
    .execute(&mut tx)
    .await?;

    query(
        "
        UPDATE votes
        SET version = version + 1
        WHERE id = $1",
    )
    .bind(vote_id)
    .execute(&mut tx)
    .await?;

    query(
        "
        UPDATE questions
        SET version = version + 1
        WHERE id = $1",
    )
    .bind(qst_id)
    .execute(&mut tx)
    .await?;
    tx.commit().await?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn delete(option_id: Path<(i32,)>, db: Data<PgPool>) -> Result<HttpResponse, Error> {
    let option_id = option_id.into_inner().0;
    let mut tx = db.begin().await?;
    let has_related_answer: bool = query_scalar("SELECT EXISTS(SELECT id FROM answers WHERE option_id = $1)").bind(option_id).fetch_one(&mut tx).await?;
    if has_related_answer {
        tx.rollback().await?;
        return Err(Error::BusinessError("there is some answer related to this option".into()));
    }
    let (oid, vid, qid): (i32, i32, i32) = query_as(
        "SELECT o.id, v.id, q.id
        FROM organizations AS o
        JOIN votes AS v ON o.id = v.organization_id
        JOIN questions AS q ON v.id = q.vote_id
        JOIN options AS op ON q.id = op.question_id
        WHERE op.id = $1
        FOR UPDATE",
    )
    .bind(option_id)
    .fetch_one(&mut tx)
    .await?;

    query("DELETE FROM options WHERE id = $1").bind(option_id).execute(&mut tx).await?;
    query("UPDATE organizations SET version = version + 1 WHERE id = $1").bind(oid).execute(&mut tx).await?;
    query("UPDATE votes SET version = version + 1 WHERE id = $1").bind(vid).execute(&mut tx).await?;
    query("UPDATE questions SET version = version + 1 WHERE id = $1").bind(qid).execute(&mut tx).await?;
    tx.commit().await?;
    Ok(HttpResponse::build(StatusCode::OK).finish())
}
