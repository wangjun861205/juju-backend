use crate::request::Pagination;
use actix_web::http::StatusCode;
use actix_web::web::Query;
use actix_web::HttpResponse;
use sqlx::QueryBuilder;

use crate::context::UserInfo;
use crate::database::sqlx::PgSqlx;
use crate::error::Error;
use crate::models::question::Create as QuestionCreate;
use crate::{
    actix_web::web::{Data, Json, Path},
    response::{CreateResponse, DeleteResponse},
};

use crate::core::db::TxStorer;
use crate::core::{
    option::options_of_question,
    question::{create_question, delete_question, question_detail},
};
use crate::models::{
    option::Opt,
    question::{Question, QuestionType},
};
use crate::response::List;
use crate::serde::{Deserialize, Serialize};
use crate::sqlx::{query, query_as, query_scalar, FromRow, PgPool};

#[derive(Debug, Deserialize)]
pub struct QuestionInsertion {
    pub description: String,
    pub type_: QuestionType,
}

pub async fn create(user_info: UserInfo, vote_id: Path<(i32,)>, Json(question): Json<QuestionCreate>, db: Data<PgPool>) -> Result<Json<CreateResponse>, Error> {
    let mut storer = PgSqlx::new(db.begin().await?);
    let id = create_question(user_info.id, vote_id.0, &mut storer, question).await?;
    storer.commit().await?;
    Ok(Json(CreateResponse { id }))
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

#[derive(Debug, Serialize)]
pub struct QuestionDetail {
    id: i32,
    description: String,
    type_: QuestionType,
    opts: Vec<Opt>,
}

pub async fn detail(user_info: UserInfo, qst_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<Question>, Error> {
    let qst_id = qst_id.into_inner().0;
    let tx = db.begin().await?;
    let mut storer = PgSqlx::new(tx);
    let question = question_detail(&mut storer, user_info.id, qst_id).await?;
    storer.commit().await?;
    Ok(Json(question))
}

pub async fn delete(user_info: UserInfo, qst_id: Path<(i32,)>, db: Data<PgPool>) -> Result<HttpResponse, Error> {
    let mut storer = PgSqlx::new(db.begin().await?);
    delete_question(&mut storer, user_info.id, qst_id.0).await?;
    Ok(HttpResponse::new(StatusCode::OK))
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
            JOIN organization_members AS uo ON u.id = uo.user_id
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
        JOIN organization_members AS uo ON u.id = uo.user_id
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
                    images: Vec::new(),
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
                images: Vec::new(),
            }],
        });
        l
    });
    Ok(Json(List::new(list, total)))
}

pub async fn answers(user_info: UserInfo, question_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<Vec<i32>>, Error> {
    let question_id = question_id.into_inner().0;
    let answers: Vec<(Option<i32>,)> = query_as(
        "
    SELECT
        a.option_id AS option_id
    FROM users AS u
    JOIN organization_members AS uo ON u.id = uo.user_id
    JOIN votes AS v ON uo.organization_id = v.organization_id
    JOIN questions AS q ON v.id = q.vote_id
    JOIN options AS o ON q.id = o.question_id
    LEFT JOIN answers AS a ON a.user_id = u.id AND o.id = a.option_id
    WHERE u.id = $1 AND q.id = $2",
    )
    .bind(user_info.id)
    .bind(question_id)
    .fetch_all(&mut db.acquire().await?)
    .await?;
    let res = answers.into_iter().filter_map(|a| a.0.map(|id| id)).collect();
    Ok(Json(res))
}

pub async fn options(question_id: Path<(i32,)>, db: Data<PgPool>) -> Result<Json<List<Opt>>, Error> {
    let mut storer = PgSqlx::new(db.begin().await?);
    let (opts, total) = options_of_question(&mut storer, question_id.0).await?;
    Ok(Json(List::new(opts, total)))
}
