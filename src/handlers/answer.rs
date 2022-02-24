use crate::actix_web::{
    http::StatusCode,
    web::{Json, Path},
    HttpResponse,
};
use crate::context::UserInfo;
use crate::diesel::{
    dsl::{date, exists, now, sql_query},
    pg::types::sql_types::Array,
    select,
    sql_types::{Bool, Integer},
    BoolExpressionMethods, Connection, ExpressionMethods, NullableExpressionMethods, QueryDsl, RunQueryDsl,
};
use crate::error::Error;
use crate::handlers::DB;
use crate::models::{AnswerInsertion, QuestionType};
use crate::schema::{answers, options, organizations, questions, users, users_organizations, votes};
use crate::serde::Serialize;

#[derive(Debug, QueryableByName)]
pub struct AnswerValidResult {
    #[sql_type = "Bool"]
    is_valid: bool,
}

pub async fn submit_answer(user_info: UserInfo, Path((qst_id,)): Path<(i32,)>, Json(answer): Json<Vec<i32>>, db: DB) -> Result<HttpResponse, Error> {
    let conn = db.get()?;
    conn.transaction::<_, Error, _>(|| {
        let is_vote_valid = select(exists(
            options::table
                .inner_join(questions::table.inner_join(votes::table.inner_join(organizations::table.inner_join(users_organizations::table))))
                .filter(
                    questions::dsl::id
                        .eq(qst_id)
                        .and(votes::deadline.is_null().or(votes::deadline.gt(date(now).nullable())))
                        .and(users_organizations::dsl::user_id.eq(user_info.id)),
                ),
        ))
        .get_result::<bool>(&conn)?;
        if !is_vote_valid {
            return Err(Error::BusinessError("vote not exists or invalid vote status".into()));
        }
        let answer_valid_query = sql_query(
            r#" select (ids @> $1) as is_valid from (
                select q.id, array_agg(o.id) as ids
                from questions as q join options as o on q.id = o.question_id
                where q.id = $2 
                group by q.id
            ) as t"#,
        );
        let is_answer_valid = answer_valid_query
            .bind::<Array<Integer>, _>(answer.clone())
            .bind::<Integer, _>(qst_id)
            .get_result::<AnswerValidResult>(&conn)?;
        if !is_answer_valid.is_valid {
            return Err(Error::BusinessError("invalid answer".into()));
        }
        diesel::delete(answers::table)
            .filter(
                answers::id
                    .eq_any(
                        questions::table
                            .inner_join(options::table.inner_join(answers::table))
                            .filter(questions::id.eq(qst_id))
                            .select(answers::id),
                    )
                    .and(answers::user_id.eq(user_info.id)),
            )
            .execute(&conn)?;

        diesel::insert_into(answers::table)
            .values(answer.into_iter().map(|o| AnswerInsertion { user_id: user_info.id, option_id: o }).collect::<Vec<_>>())
            .execute(&conn)?;
        Ok(())
    })?;
    Ok(HttpResponse::build(StatusCode::OK).finish())
}

#[derive(Debug, Serialize, Queryable)]
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

pub async fn answer_list(user_info: UserInfo, Path((qst_id,)): Path<(i32,)>, db: DB) -> Result<Json<AnswerList>, Error> {
    let conn = db.get()?;
    let resp = conn.transaction::<AnswerList, Error, _>(|| {
        let question_type = users::table
            .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table.inner_join(questions::table))))
            .filter(users::id.eq(user_info.id).and(questions::id.eq(qst_id)))
            .select(questions::type_)
            .get_result::<QuestionType>(&conn)?;
        let options: Vec<OptionItem> = questions::table
            .inner_join(options::table)
            .filter(questions::id.eq(qst_id))
            .select((options::id, options::option))
            .load(&conn)?;

        let answers = questions::table
            .inner_join(options::table.inner_join(answers::table))
            .filter(answers::user_id.eq(user_info.id).and(questions::id.eq(qst_id)))
            .select(options::id)
            .load::<i32>(&conn)?;
        Ok(AnswerList {
            question_type: question_type,
            options: options,
            answers: answers,
        })
    })?;
    Ok(Json(resp))
}
