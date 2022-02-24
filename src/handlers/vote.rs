use crate::actix_web::web::{Json, Path, Query};
use crate::chrono::{DateTime, Local, NaiveDate, Utc};
use crate::context::UserInfo;
use crate::diesel::{
    delete,
    dsl::{exists, sql, sql_query, update as update_},
    insert_into, select,
    sql_types::*,
    BelongingToDsl, BoolExpressionMethods, Connection, ExpressionMethods, QueryDsl, RunQueryDsl,
};
use crate::error::Error;
use crate::handlers::DB;
use crate::models::{Date, Question, Vote, VoteInsertion, VoteReadMarkInsertion, VoteStatus, VoteUpdation as MVoteUpdation};
use crate::response::{CreateResponse, DeleteResponse, List, UpdateResponse};
use crate::schema::{organizations, questions, users, users_organizations, vote_update_marks, votes};
use crate::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Creation {
    name: String,
    deadline: Option<DateTime<Utc>>,
}

pub async fn create(user_info: UserInfo, Path((org_id,)): Path<(i32,)>, Json(body): Json<Creation>, db: DB) -> Result<Json<CreateResponse>, Error> {
    let conn = db.get()?;
    let id = conn.transaction::<_, Error, _>(|| {
        let exists = select(exists(
            users_organizations::table
                .filter(users_organizations::user_id.eq(user_info.id))
                .inner_join(organizations::table)
                .filter(organizations::id.eq(org_id)),
        ))
        .get_result::<bool>(&conn)?;
        if !exists {
            return Err(Error::BusinessError("irrelative organization".into()));
        }
        let vote_id = insert_into(votes::table)
            .values(VoteInsertion {
                name: body.name,
                deadline: if let Some(dl) = body.deadline { Some(dl.naive_utc().date()) } else { None },
                organization_id: org_id,
            })
            .execute(&conn)?;
        let user_ids: Vec<i32> = users::table
            .inner_join(users_organizations::table.inner_join(organizations::table))
            .filter(organizations::id.eq(org_id))
            .select(users::id)
            .load(&conn)?;
        insert_into(vote_update_marks::table)
            .values(
                user_ids
                    .into_iter()
                    .map(|id| VoteReadMarkInsertion {
                        user_id: id,
                        vote_id: vote_id as i32,
                        has_updated: true,
                    })
                    .collect::<Vec<VoteReadMarkInsertion>>(),
            )
            .execute(&conn)?;
        Ok(vote_id)
    })?;
    return Ok(Json(CreateResponse { id: id }));
}

#[derive(Debug, Clone, Deserialize)]
pub struct VoteUpdation {
    name: String,
    deadline: Option<NaiveDate>,
}

pub async fn update(user_info: UserInfo, Path((vote_id,)): Path<(i32,)>, Json(VoteUpdation { name, deadline }): Json<VoteUpdation>, db: DB) -> Result<Json<UpdateResponse>, Error> {
    let conn = db.get()?;
    let updated: usize = conn.transaction::<_, Error, _>(|| {
        let mut vote: MVoteUpdation = users::table
            .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table)))
            .filter(users::id.eq(user_info.id).and(votes::id.eq(vote_id)))
            .select((votes::name, votes::deadline))
            .for_update()
            .first(&conn)?;
        update_(vote_update_marks::table)
            .filter(vote_update_marks::vote_id.eq(vote_id))
            .set(vote_update_marks::has_updated.eq(true))
            .execute(&conn)?;
        vote.name = name;
        vote.deadline = deadline;
        let updated = update_(votes::table).filter(votes::id.eq(vote_id)).set(vote).execute(&conn)?;
        Ok(updated)
    })?;
    Ok(Json(UpdateResponse { updated }))
}

#[derive(Debug, Serialize, Queryable)]
pub struct Item {
    id: i32,
    name: String,
    deadline: Option<NaiveDate>,
    status: String,
    has_updated: bool,
}

#[derive(Debug, Serialize)]
struct VoteDetail {
    vote: Vote,
    dates: Vec<Date>,
    questions: Vec<Question>,
}

#[derive(Debug, Deserialize)]
pub struct VoteParam {
    page: i64,
    size: i64,
}

pub async fn list(user_info: UserInfo, param: Query<VoteParam>, organization_id: Path<(i32,)>, db: DB) -> Result<Json<List<Item>>, Error> {
    let conn = db.get()?;
    let (votes, total) = conn.transaction::<(Vec<Item>, i64), Error, _>(|| {
        let total: i64 = users::table
            .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table)))
            .filter(users::id.eq(user_info.id).and(organizations::id.eq(organization_id.0 .0)))
            .count()
            .get_result(&conn)?;
        let votes: Vec<Item> = users::table
            .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table.inner_join(vote_update_marks::table))))
            .select((
                votes::id,
                votes::name,
                votes::deadline,
                sql::<Text>("CASE WHEN votes.deadline < DATE(NOW()) THEN 'Closed' ELSE 'Collecting' END"),
                vote_update_marks::has_updated,
            ))
            .filter(
                users::id
                    .eq(user_info.id)
                    .and(organizations::id.eq(organization_id.0 .0))
                    .and(vote_update_marks::user_id.eq(user_info.id)),
            )
            .offset((param.page - 1) * param.size)
            .limit(param.size)
            .load::<Item>(&conn)?;
        Ok((votes, total))
    })?;
    Ok(Json(List::new(votes, total)))
}

#[derive(Debug, Serialize)]
pub struct Detail {
    id: i32,
    name: String,
    deadline: Option<NaiveDate>,
    status: VoteStatus,
}

pub async fn detail(user_info: UserInfo, Path((vote_id,)): Path<(i32,)>, db: DB) -> Result<Json<Detail>, Error> {
    let conn = db.get()?;
    let vote = conn.transaction::<_, Error, _>(|| {
        let vote: Vote = users::table
            .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table)))
            .filter(users::dsl::id.eq(user_info.id).and(votes::dsl::id.eq(vote_id)))
            .select(votes::all_columns)
            .for_update()
            .get_result(&conn)?;
        update_(vote_update_marks::table)
            .filter(vote_update_marks::user_id.eq(user_info.id).and(vote_update_marks::vote_id.eq(vote_id)))
            .set(vote_update_marks::has_updated.eq(false))
            .execute(&conn)?;
        Ok(vote)
    })?;
    Ok(Json(Detail {
        id: vote.id,
        name: vote.name,
        deadline: vote.deadline.clone(),
        status: if let Some(dl) = &vote.deadline {
            if dl < &Utc::today().naive_utc() {
                VoteStatus::Closed
            } else {
                VoteStatus::Collecting
            }
        } else {
            VoteStatus::Collecting
        },
    }))
}

#[derive(Debug, Clone, Serialize, QueryableByName)]
pub struct OptionReport {
    #[sql_type = "Text"]
    option: String,
    #[sql_type = "Integer"]
    percentage: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct QuestionReport {
    question: String,
    options: Vec<OptionReport>,
}

fn gen_question_report(question_id: i32, db: &DB) -> Result<QuestionReport, Error> {
    let conn = &db.get()?;
    let question = questions::table.find(question_id).get_result::<Question>(conn)?;
    let stmt = r#"
    select o.option as option, (count(distinct a.id)::float / (count(distinct uo.user_id))::float * 10000)::int as percentage
    from users_organizations as uo
    join organizations as org on uo.organization_id = org.id
    join votes as v on org.id = v.organization_id
    join questions as q on v.id = q.vote_id
    join options as o on q.id = o.question_id
    left join answers as a on o.id = a.option_id
    where q.id = $1
    group by option"#;
    let opts = sql_query(stmt).bind::<Integer, _>(question_id).load::<OptionReport>(conn)?;
    Ok(QuestionReport {
        question: question.description,
        options: opts,
    })
}

pub async fn question_reports(user_info: UserInfo, Path((vote_id,)): Path<(i32,)>, db: DB) -> Result<Json<Vec<QuestionReport>>, Error> {
    let qids: Vec<i32> = users::table
        .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table.inner_join(questions::table))))
        .filter(users::id.eq(user_info.id).and(votes::id.eq(vote_id)))
        .select(questions::id)
        .load(&db.get()?)?;
    let resports: Vec<QuestionReport> = qids.into_iter().map(|id| gen_question_report(id, &db)).collect::<Result<Vec<QuestionReport>, Error>>()?;
    Ok(Json(resports))
}

pub async fn delete_vote(user_info: UserInfo, Path((vote_id,)): Path<(i32,)>, db: DB) -> Result<Json<DeleteResponse>, Error> {
    let deleted: usize = delete(votes::table)
        .filter(
            votes::id
                .eq_any(
                    users::table
                        .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table)))
                        .filter(users::id.eq(user_info.id))
                        .select(votes::id),
                )
                .and(votes::id.eq(vote_id)),
        )
        .execute(&db.get()?)?;
    Ok(Json(DeleteResponse::new(deleted)))
}
