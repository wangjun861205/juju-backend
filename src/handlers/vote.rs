use crate::actix_web::{
    http::StatusCode,
    web::{Json, Path, Query},
    HttpResponse,
};
use crate::chrono::NaiveDate;
use crate::context::UserInfo;
use crate::diesel::{
    delete,
    dsl::{exists, sql_query},
    insert_into, select,
    sql_types::*,
    BelongingToDsl, BoolExpressionMethods, Connection, ExpressionMethods, QueryDsl, RunQueryDsl,
};
use crate::error::Error;
use crate::handlers::DB;
use crate::models::{Date, Question, Vote, VoteInsertion, VoteStatus};
use crate::response::DeleteResponse;
use crate::response::List;
use crate::schema::{organizations, questions, users, users_organizations, votes};
use crate::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Creation {
    name: String,
    deadline: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreationResponse {
    id: usize,
}

pub async fn create(user_info: UserInfo, Path((org_id,)): Path<(i32,)>, body: Json<Creation>, db: DB) -> Result<Json<CreationResponse>, Error> {
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
        Ok(insert_into(votes::table)
            .values(VoteInsertion {
                name: body.0.name,
                deadline: body.0.deadline,
                status: VoteStatus::Collecting,
                organization_id: org_id,
            })
            .execute(&conn)?)
    })?;
    return Ok(Json(CreationResponse { id: id }));
}

#[derive(Debug, Clone, Deserialize)]
pub struct VoteUpdation {
    name: String,
    deadline: Option<String>,
}

pub async fn update_vote(user_info: UserInfo, Path((org_id, vote_id)): Path<(i32, i32)>, vote: Json<VoteUpdation>, db: DB) -> Result<HttpResponse, Error> {
    use crate::models;
    let deadline = if let Some(dl) = vote.clone().deadline {
        Some(NaiveDate::parse_from_str(&dl, "%Y-%m-%d")?)
    } else {
        None
    };
    let status = if let Some(d) = &deadline {
        if d < &chrono::Local::today().naive_local() {
            models::VoteStatus::Closed
        } else {
            models::VoteStatus::Collecting
        }
    } else {
        models::VoteStatus::Collecting
    };
    diesel::update(votes::table)
        .filter(
            votes::dsl::organization_id
                .eq_any(
                    users::table
                        .inner_join(users_organizations::table.inner_join(organizations::table))
                        .select(organizations::id)
                        .filter(users::dsl::id.eq(user_info.id).and(organizations::dsl::id.eq(org_id))),
                )
                .and(votes::dsl::id.eq(vote_id)),
        )
        .set(models::VoteUpdation {
            name: vote.clone().name,
            deadline: deadline,
            status: status,
        })
        .execute(&db.get()?)?;
    Ok(HttpResponse::build(StatusCode::OK).finish())
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

pub async fn vote_list(user_info: UserInfo, param: Query<VoteParam>, organization_id: Path<(i32,)>, db: DB) -> Result<HttpResponse, Error> {
    let conn = db.get()?;
    let (votes, total) = conn.transaction::<(Vec<Vote>, i64), Error, _>(|| {
        let total: i64 = users::table
            .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table)))
            .filter(users::id.eq(user_info.id).and(organizations::id.eq(organization_id.0 .0)))
            .count()
            .get_result(&conn)?;
        let votes = users::table
            .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table)))
            .select(votes::all_columns)
            .filter(users::id.eq(user_info.id).and(organizations::id.eq(organization_id.0 .0)))
            .offset((param.page - 1) * param.size)
            .limit(param.size)
            .load::<Vote>(&conn)?;
        Ok((votes, total))
    })?;
    return Ok(HttpResponse::build(StatusCode::OK).json(List::new(votes, total)));
}

pub async fn vote_detail(user_info: UserInfo, Path((vote_id,)): Path<(i32,)>, db: DB) -> Result<HttpResponse, Error> {
    let conn = db.get()?;
    let detail = conn.transaction::<VoteDetail, Error, _>(|| {
        let vote: Vote = users::table
            .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table)))
            .filter(users::dsl::id.eq(user_info.id).and(votes::dsl::id.eq(vote_id)))
            .select(votes::all_columns)
            .get_result(&conn)?;
        let dates: Vec<Date> = Date::belonging_to(&vote).load(&conn)?;
        let questions: Vec<Question> = Question::belonging_to(&vote).load(&conn)?;
        Ok(VoteDetail {
            vote: vote,
            dates: dates,
            questions: questions,
        })
    })?;
    Ok(HttpResponse::build(StatusCode::OK).json(detail))
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
