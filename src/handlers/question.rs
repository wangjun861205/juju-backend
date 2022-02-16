use crate::actix_web::{
    http::StatusCode,
    web::{Json, Path},
    HttpResponse,
};
use crate::context::UserInfo;
use crate::diesel::{
    dsl::{exists, select, sql},
    sql_types::*,
    BelongingToDsl, BoolExpressionMethods, Connection, ExpressionMethods, GroupByDsl, QueryDsl, RunQueryDsl,
};
use crate::error::Error;
use crate::handlers::DB;
use crate::schema::*;

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

pub async fn create_question(user_info: UserInfo, Path((vote_id,)): Path<(i32,)>, Json(body): Json<CreateRequest>, db: DB) -> Result<HttpResponse, Error> {
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
        diesel::insert_into(options::table).values(opts).execute(&conn)?;
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
}

pub async fn question_list(user_info: UserInfo, Path((vote_id,)): Path<(i32,)>, db: DB) -> Result<Json<List<QuestionListResponse>>, Error> {
    let list = users::table
        .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table.inner_join(questions::table.inner_join(options::table.left_join(answers::table))))))
        .select((questions::id, questions::description, sql::<diesel::sql_types::Bool>("count(distinct answers.id) > 0")))
        .group_by((questions::id, questions::description, questions::type_))
        .filter(users::id.eq(user_info.id).and(votes::id.eq(vote_id)))
        .load::<(i32, String, bool)>(&db.get()?)?;
    Ok(Json(List::new(
        list.into_iter()
            .map(|q| QuestionListResponse {
                id: q.0,
                description: q.1,
                has_answered: q.2,
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

pub async fn question_detail(user_info: UserInfo, Path((qst_id,)): Path<(i32,)>, db: DB) -> Result<Json<QuestionDetail>, Error> {
    let qst: models::Question = users::table
        .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table.inner_join(questions::table))))
        .filter(users::id.eq(user_info.id).and(questions::id.eq(qst_id)))
        .select(questions::all_columns)
        .get_result(&db.get()?)?;
    let opts = models::Opt::belonging_to(&qst).load(&db.get()?)?;
    Ok(Json(QuestionDetail {
        id: qst.id,
        description: qst.description,
        type_: qst.type_,
        opts: opts,
    }))
}
