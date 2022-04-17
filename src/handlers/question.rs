use crate::context::UserInfo;
use crate::diesel::{
    dsl::{delete as delete_, exists, insert_into, select, sql, update},
    sql_types::*,
    BelongingToDsl, BoolExpressionMethods, Connection, ExpressionMethods, GroupByDsl, QueryDsl, RunQueryDsl,
};
use crate::error::Error;
use crate::handlers::DB;
use crate::schema::*;
use crate::{
    actix_web::web::{Json, Path},
    response::{CreateResponse, DeleteResponse},
};

use crate::models::{Opt, OptInsertion, Question, QuestionType};
use crate::response::List;
use crate::serde::{Deserialize, Serialize};

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

#[derive(Debug, QueryableByName)]
pub struct UserID {
    #[sql_type = "Integer"]
    id: i32,
}

pub async fn create(
    user_info: UserInfo,
    vote_id: Path<(i32,)>,
    Json(CreateRequest {
        question: QuestionInsertion { description, type_ },
        options,
    }): Json<CreateRequest>,
    db: DB,
) -> Result<Json<CreateResponse>, Error> {
    let vote_id = vote_id.into_inner().0;
    let conn = db.get()?;
    let id = conn.transaction::<_, Error, _>(|| {
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
            .values((
                questions::description.eq(description),
                questions::type_.eq(type_),
                questions::vote_id.eq(vote_id),
                questions::version.eq(1),
            ))
            .returning(questions::id)
            .get_result::<i32>(&conn)?;
        let opts: Vec<OptInsertion> = options.into_iter().map(|v| OptInsertion { question_id: qid, option: v }).collect();
        insert_into(options::table).values(opts).execute(&conn)?;
        insert_into(question_read_marks::table)
            .values((
                question_read_marks::question_id.eq(qid),
                question_read_marks::user_id.eq(user_info.id),
                question_read_marks::version.eq(1),
            ))
            .execute(&conn)?;
        Ok(qid)
    })?;
    return Ok(Json(CreateResponse { id: id }));
}

#[derive(Debug, Serialize, Queryable)]
pub struct Item {
    id: i32,
    description: String,
    type_: QuestionType,
    version: i64,
    has_answered: bool,
    has_updated: bool,
}

pub async fn list(user_info: UserInfo, vote_id: Path<(i32,)>, db: DB) -> Result<Json<List<Item>>, Error> {
    let vote_id = vote_id.into_inner().0;
    let conn = db.get()?;
    let list = conn.transaction::<Vec<Item>, Error, _>(|| {
        let list = users::table
            .inner_join(
                users_organizations::table
                    .inner_join(organizations::table.inner_join(votes::table.inner_join(questions::table.left_join(options::table.left_join(answers::table)).inner_join(question_read_marks::table)))),
            )
            .select((
                questions::id,
                questions::description,
                questions::type_,
                questions::version,
                sql::<diesel::sql_types::Bool>("count(distinct answers.id) > 0"),
                sql::<Bool>("questions.version > SUM(question_read_marks.version)"),
            ))
            .group_by((questions::id, questions::description, questions::type_, questions::version))
            .filter(users::id.eq(user_info.id).and(votes::id.eq(vote_id)).and(question_read_marks::user_id.eq(user_info.id)))
            .load::<Item>(&conn)?;
        Ok(list)
    })?;
    Ok(Json(List::new(list, 0)))
}

#[derive(Debug, Serialize)]
pub struct QuestionDetail {
    id: i32,
    description: String,
    type_: QuestionType,
    opts: Vec<Opt>,
}

pub async fn detail(user_info: UserInfo, qst_id: Path<(i32,)>, db: DB) -> Result<Json<QuestionDetail>, Error> {
    let qst_id = qst_id.into_inner().0;
    let qst: Question = users::table
        .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table.inner_join(questions::table))))
        .filter(users::id.eq(user_info.id).and(questions::id.eq(qst_id)))
        .select(questions::all_columns)
        .for_share()
        .get_result(&db.get()?)?;
    let opts = Opt::belonging_to(&qst).load(&db.get()?)?;
    update(question_read_marks::table)
        .filter(question_read_marks::user_id.eq(user_info.id).and(question_read_marks::question_id.eq(qst_id)))
        .set(question_read_marks::version.eq(qst.version))
        .execute(&db.get()?)?;
    Ok(Json(QuestionDetail {
        id: qst.id,
        description: qst.description,
        type_: qst.type_,
        opts: opts,
    }))
}

pub async fn delete(user_info: UserInfo, qst_id: Path<(i32,)>, db: DB) -> Result<Json<DeleteResponse>, Error> {
    let qst_id = qst_id.into_inner().0;
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
