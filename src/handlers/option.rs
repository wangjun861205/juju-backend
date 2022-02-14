use crate::actix_web::{
    http::StatusCode,
    web::{Json, Path, Query},
    HttpResponse,
};
use crate::context::UserInfo;
use crate::diesel::{dsl::exists, BoolExpressionMethods, Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use crate::error::Error;
use crate::handlers::DB;
use crate::models::{Opt, OptInsertion};
use crate::response::List;
use crate::schema::{options, organizations, questions, users, users_organizations, votes};
use crate::serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Param {
    page: i64,
    size: i64,
}

pub async fn option_list(user_info: UserInfo, Path((qst_id,)): Path<(i32,)>, Query(param): Query<Param>, db: DB) -> Result<Json<List<Opt>>, Error> {
    let conn = db.get()?;
    let (list, total) = conn.transaction::<(Vec<Opt>, i64), Error, _>(|| {
        let query = users::table
            .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table.inner_join(questions::table.inner_join(options::table)))))
            .filter(users::id.eq(user_info.id).and(questions::id.eq(qst_id)));
        let total: i64 = query.count().get_result(&conn)?;
        let list: Vec<Opt> = query.select(options::all_columns).limit(param.size).offset((param.page - 1) * param.size).load(&conn)?;
        Ok((list, total))
    })?;
    Ok(Json(List::new(list, total)))
}

#[derive(Debug, Clone, Deserialize)]
pub struct OptAdd {
    pub option: String,
}

pub async fn add_opts(user_info: UserInfo, Path((qst_id,)): Path<(i32,)>, body: Json<Vec<OptAdd>>, db: DB) -> Result<HttpResponse, Error> {
    let conn = db.get()?;
    conn.transaction::<(), Error, _>(|| {
        let question_exists: bool = diesel::select(exists(
            users::table
                .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table.inner_join(questions::table))))
                .filter(users::id.eq(user_info.id).and(questions::id.eq(qst_id))),
        ))
        .for_update()
        .get_result(&conn)?;
        if !question_exists {
            return Err(Error::BusinessError("question not exist".into()));
        }
        diesel::insert_into(options::table)
            .values::<Vec<OptInsertion>>(
                body.0
                    .into_iter()
                    .map(|o| OptInsertion {
                        question_id: qst_id,
                        option: o.option,
                    })
                    .collect(),
            )
            .execute(&conn)?;

        Ok(())
    })?;
    Ok(HttpResponse::build(StatusCode::OK).finish())
}
