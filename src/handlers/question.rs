use crate::context::UserInfo;
use crate::diesel::{
    dsl::{delete as delete_, exists, insert_into, select, sql, sql_query, update},
    sql_types::*,
    BelongingToDsl, BoolExpressionMethods, Connection, ExpressionMethods, GroupByDsl, QueryDsl, RunQueryDsl,
};
use crate::error::Error;
use crate::handlers::DB;
use crate::schema::*;
use crate::{
    actix_web::{
        http::StatusCode,
        web::{Json, Path},
        HttpResponse,
    },
    response::DeleteResponse,
};

use crate::models;
use crate::response::List;
use crate::serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Question {
    pub description: String,
    pub type_: models::QuestionType,
}

#[derive(Debug, Deserialize)]
pub struct CreateRequest {
    pub question: Question,
    pub options: Vec<String>,
}

#[derive(Debug, QueryableByName)]
pub struct UserID {
    #[sql_type = "Integer"]
    id: i32,
}

pub async fn create(user_info: UserInfo, Path((vote_id,)): Path<(i32,)>, Json(body): Json<CreateRequest>, db: DB) -> Result<HttpResponse, Error> {
    let conn = db.get()?;
    conn.transaction::<_, Error, _>(|| {
        let exists = select(exists(
            users_organizations::table
                .inner_join(organizations::table.inner_join(votes::table))
                .filter(users_organizations::user_id.eq(user_info.id).and(votes::id.eq(vote_id))),
        ))
        .get_result::<bool>(&conn)?;
        if !exists {
            return Err(Error::BusinessError("irrelative vote or vote not exists".into()));
        }
        let qid = diesel::insert_into(questions::table)
            .values(models::QuestionInsertion {
                description: body.question.description,
                type_: body.question.type_,
                vote_id: vote_id,
            })
            .returning(questions::id)
            .get_result::<i32>(&conn)?;
        let opts: Vec<models::OptInsertion> = body.options.into_iter().map(|v| models::OptInsertion { question_id: qid, option: v }).collect();
        insert_into(options::table).values(opts).execute(&conn)?;
        insert_into(question_update_marks::table)
            .values(
                sql_query(
                    r#"
        SELECT 
            u2.id, 
        FROM users AS u1 
        JOIN users_organizations AS uo1 ON u1.id = uo.user_id 
        JOIN organizations AS o ON uo1.organization_id = o.id
        JOIN users_organizations AS uo2 ON o.id = uo2.organization_id
        JOIN users AS u2 ON uo.user_id = u2.id
        WHERE u1.id = $1
        "#,
                )
                .bind::<Integer, _>(user_info.id)
                .load::<UserID>(&conn)?
                .into_iter()
                .map(|UserID { id }| {
                    (
                        question_update_marks::user_id.eq(id),
                        question_update_marks::question_id.eq(qid),
                        question_update_marks::has_updated.eq(true),
                    )
                })
                .collect::<Vec<(_, _, _)>>(),
            )
            .execute(&conn)?;

        Ok(())
    })?;
    return Ok(HttpResponse::build(StatusCode::OK).finish());
}

#[derive(Debug, Serialize, QueryableByName)]
pub struct QuestionListResponse {
    #[sql_type = "Integer"]
    id: i32,
    #[sql_type = "Text"]
    description: String,
    #[sql_type = "Bool"]
    has_answered: bool,
    #[sql_type = "Bool"]
    has_updated: bool,
}

pub async fn list(user_info: UserInfo, Path((vote_id,)): Path<(i32,)>, db: DB) -> Result<Json<List<QuestionListResponse>>, Error> {
    let list = users::table
        .inner_join(
            users_organizations::table
                .inner_join(organizations::table.inner_join(votes::table.inner_join(questions::table.inner_join(options::table.left_join(answers::table)).inner_join(question_update_marks::table)))),
        )
        .select((
            questions::id,
            questions::description,
            sql::<diesel::sql_types::Bool>("count(distinct answers.id) > 0"),
            question_update_marks::has_updated,
        ))
        .group_by((questions::id, questions::description, questions::type_, question_update_marks::has_updated))
        .filter(users::id.eq(user_info.id).and(votes::id.eq(vote_id)).and(question_update_marks::user_id.eq(user_info.id)))
        .load::<(i32, String, bool, bool)>(&db.get()?)?;
    Ok(Json(List::new(
        list.into_iter()
            .map(|q| QuestionListResponse {
                id: q.0,
                description: q.1,
                has_answered: q.2,
                has_updated: q.3,
            })
            .collect(),
        0,
    )))
}

#[derive(Debug, Serialize)]
pub struct QuestionDetail {
    id: i32,
    description: String,
    type_: models::QuestionType,
    opts: Vec<models::Opt>,
}

pub async fn detail(user_info: UserInfo, Path((qst_id,)): Path<(i32,)>, db: DB) -> Result<Json<QuestionDetail>, Error> {
    let qst: models::Question = users::table
        .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table.inner_join(questions::table))))
        .filter(users::id.eq(user_info.id).and(questions::id.eq(qst_id)))
        .select(questions::all_columns)
        .get_result(&db.get()?)?;
    let opts = models::Opt::belonging_to(&qst).load(&db.get()?)?;
    update(question_update_marks::table)
        .filter(question_update_marks::user_id.eq(user_info.id).and(question_update_marks::question_id.eq(qst_id)))
        .set(question_update_marks::has_updated.eq(false))
        .execute(&db.get()?)?;
    Ok(Json(QuestionDetail {
        id: qst.id,
        description: qst.description,
        type_: qst.type_,
        opts: opts,
    }))
}

pub async fn delete(user_info: UserInfo, Path((qst_id,)): Path<(i32,)>, db: DB) -> Result<Json<DeleteResponse>, Error> {
    let deleted = delete_(questions::table)
        .filter(
            questions::id
                .eq_any(
                    users::table
                        .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table.inner_join(questions::table))))
                        .filter(users::id.eq(user_info.id))
                        .select(questions::id),
                )
                .and(questions::id.eq(qst_id)),
        )
        .execute(&db.get()?)?;
    Ok(Json(DeleteResponse::new(deleted)))
}
