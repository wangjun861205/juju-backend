use crate::context::UserInfo;
use crate::diesel::{
    dsl::{delete as delete_, insert_into, sql, update as update_},
    sql_types::Bool,
    BoolExpressionMethods, Connection, ExpressionMethods, QueryDsl, RunQueryDsl,
};
use crate::error::Error;
use crate::handlers::DB;
use crate::models::OptInsertion;
use crate::response::{CreateResponse, DeleteResponse};
use crate::schema::{answers, options, organizations, questions, users, users_organizations, votes};
use crate::serde::Deserialize;
use crate::serde::Serialize;
use crate::{
    actix_web::web::{Json, Path},
    models::QuestionType,
};
use std::ops::Add;

#[derive(Debug, Serialize)]
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

pub async fn list(user_info: UserInfo, qst_id: Path<(i32,)>, db: DB) -> Result<Json<ListResponse>, Error> {
    let qst_id = qst_id.into_inner().0;
    let conn = db.get()?;
    let res = conn.transaction::<ListResponse, Error, _>(|| {
        let question_type: QuestionType = users::table
            .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table.inner_join(questions::table))))
            .select(questions::type_)
            .filter(users::id.eq(user_info.id).and(questions::id.eq(qst_id)))
            .get_result(&conn)?;
        let items: Vec<(i32, String, bool)> = users::table
            .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table.inner_join(questions::table.inner_join(options::table.left_join(answers::table))))))
            .select((options::id, options::option, sql::<Bool>("answers.id IS NOT NULL")))
            .filter(
                users::id
                    .eq(user_info.id)
                    .and(questions::id.eq(qst_id))
                    .and(answers::user_id.is_null().or(answers::user_id.eq(user_info.id))),
            )
            .order_by(options::id)
            .load(&conn)?;
        Ok(ListResponse {
            question_type: question_type,
            items: items.into_iter().map(|(id, option, checked)| Item { id, option, checked }).collect(),
        })
    })?;
    Ok(Json(res))
}

#[derive(Debug, Clone, Deserialize)]
pub struct OptAdd {
    pub option: String,
}

pub async fn add_opts(user_info: UserInfo, qst_id: Path<(i32,)>, body: Json<Vec<String>>, db: DB) -> Result<Json<CreateResponse>, Error> {
    let qst_id = qst_id.into_inner().0;
    let conn = db.get()?;
    let id = conn.transaction::<_, Error, _>(|| {
        let (org_id, vote_id): (i32, i32) = users::table
            .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table.inner_join(questions::table))))
            .filter(users::id.eq(user_info.id).and(questions::id.eq(qst_id)))
            .select((organizations::id, votes::id))
            .for_update()
            .first(&conn)?;
        let id = insert_into(options::table)
            .values::<Vec<OptInsertion>>(body.0.into_iter().map(|o| OptInsertion { question_id: qst_id, option: o }).collect())
            .returning(options::id)
            .get_result::<i32>(&conn)?;
        update_(organizations::table)
            .filter(organizations::id.eq(org_id))
            .set(organizations::version.eq(organizations::version.add(1)))
            .execute(&conn)?;
        update_(votes::table).filter(votes::id.eq(vote_id)).set(votes::version.eq(votes::version.add(1))).execute(&conn)?;
        update_(questions::table)
            .filter(questions::id.eq(qst_id))
            .set(questions::version.eq(questions::version.add(1)))
            .execute(&conn)?;
        Ok(id)
    })?;
    Ok(Json(CreateResponse { id: id }))
}

pub async fn delete(user_info: UserInfo, option_id: Path<(i32,)>, db: DB) -> Result<Json<DeleteResponse>, Error> {
    let option_id = option_id.into_inner().0;
    let conn = db.get()?;
    let deleted = conn.transaction::<_, Error, _>(|| {
        let (oid, vid, qid): (i32, i32, i32) = organizations::table
            .inner_join(votes::table.inner_join(questions::table.inner_join(options::table)))
            .filter(options::id.eq(option_id))
            .select((organizations::id, votes::id, options::id))
            .for_update()
            .first(&conn)?;
        let deleted = delete_(options::table)
            .filter(
                options::id
                    .eq_any(
                        users::table
                            .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table.inner_join(questions::table.inner_join(options::table)))))
                            .filter(users::id.eq(user_info.id))
                            .select(options::id),
                    )
                    .and(options::id.eq(option_id)),
            )
            .execute(&conn)?;
        if deleted == 0 {
            return Err(Error::BusinessError("option not exists".into()));
        }
        update_(organizations::table)
            .filter(organizations::id.eq(oid))
            .set(organizations::version.eq(organizations::version.add(1)))
            .execute(&conn)?;
        update_(votes::table).filter(votes::id.eq(vid)).set(votes::version.eq(votes::version.add(1))).execute(&conn)?;
        update_(questions::table)
            .filter(questions::id.eq(qid))
            .set(questions::version.eq(questions::version.add(1)))
            .execute(&conn)?;
        Ok(deleted)
    })?;
    Ok(Json(DeleteResponse { deleted }))
}
