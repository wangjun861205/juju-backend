use actix_web::http::StatusCode;
use diesel::{Connection, GroupByDsl, QueryDsl};
use rand::Rng;

use crate::actix_web::{
    cookie::Cookie,
    web::{Data, Json},
    HttpRequest, HttpResponse,
};

use crate::actix_web::web::{Path, Query};
use crate::chrono::{self, NaiveDate};
use crate::context::UserInfo;
use crate::diesel::dsl::*;
use crate::diesel::*;
use crate::diesel::{pg::PgConnection, r2d2::ConnectionManager, ExpressionMethods, RunQueryDsl};
use crate::dotenv;
use crate::error::Error;
use crate::hex::ToHex;
use crate::jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use crate::middleware::jwt::{Claim, JWT_SECRET, JWT_TOKEN};
use crate::models::*;
use crate::models::{Date, Question, Vote};
use crate::r2d2::Pool;
use crate::rand::thread_rng;
use crate::schema::users::dsl::*;
use crate::serde::{Deserialize, Serialize};
use crate::sha2::{Digest, Sha256};

use crate::schema::{answers, dates, options, organizations, questions, users, users_organizations, votes};

type DB = Data<Pool<ConnectionManager<PgConnection>>>;

#[derive(Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}

fn hash_password(pass: &str, slt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(pass);
    hasher.update(slt);
    hasher.finalize().encode_hex()
}

pub async fn login(body: Json<Login>, db: Data<Pool<ConnectionManager<PgConnection>>>) -> Result<HttpResponse, Error> {
    let conn = db.get()?;
    let l = users.filter(phone.eq(&body.0.username)).or_filter(email.eq(&body.0.username)).load::<User>(&conn)?;
    if l.is_empty() {
        return Ok(HttpResponse::build(StatusCode::FORBIDDEN).finish());
    }
    if hash_password(&body.0.password, &l[0].salt) != l[0].password {
        return Ok(HttpResponse::build(StatusCode::FORBIDDEN).finish());
    }
    let claim = Claim { uid: l[0].id };
    let secret = dotenv::var(JWT_SECRET)?;
    let token = encode(&Header::new(Algorithm::HS256), &claim, &EncodingKey::from_secret(secret.as_bytes()))?;

    Ok(HttpResponse::build(StatusCode::OK).cookie(Cookie::new(JWT_TOKEN, token)).finish())
}

fn random_salt() -> String {
    let chars = vec![
        '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
        'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ];
    let mut slt = String::new();
    let mut rng = thread_rng();
    for _ in 0..32 {
        let i = rng.gen_range(0, 61_usize);
        slt.push(chars[i]);
    }
    slt
}

#[derive(Debug, Clone, Deserialize)]
pub struct Signup {
    nickname: String,
    phone: String,
    email: String,
    password: String,
    invite_code: String,
}

pub async fn signup(body: Json<Signup>, db: Data<Pool<ConnectionManager<PgConnection>>>) -> Result<HttpResponse, Error> {
    use crate::schema::invite_codes::dsl::*;
    let conn = db.get()?;
    conn.transaction::<(), Error, _>(|| {
        let l = invite_codes.filter(code.eq(&body.0.invite_code)).for_update().load::<crate::models::InviteCode>(&conn)?;
        if l.is_empty() {
            return Err(Error::BusinessError("invalid invite code".into()));
        }
        diesel::delete(invite_codes.filter(code.eq(&body.0.invite_code))).execute(&conn)?;
        let slt = random_salt();
        let insertion = crate::models::UserInsertion {
            nickname: body.0.nickname,
            phone: body.0.phone,
            email: body.0.email,
            password: hash_password(&body.0.password, &slt),
            salt: slt,
        };
        diesel::insert_into(users).values(insertion).execute(&conn)?;
        Ok(())
    })?;
    Ok(HttpResponse::build(StatusCode::OK).finish())
}

#[derive(Debug, Clone, Deserialize)]
pub struct OrganizationCreation {
    name: String,
}

pub async fn create_organization(req: HttpRequest, body: Json<OrganizationCreation>, db: DB) -> Result<HttpResponse, Error> {
    use crate::schema::organizations::dsl::*;
    use crate::schema::users_organizations::dsl as user_org;

    if let Some(uid) = req.headers().get("UID") {
        let uid = uid.to_str()?.parse::<i32>()?;
        let conn = db.get()?;
        conn.transaction::<_, Error, _>(|| {
            let org_id = diesel::insert_into(organizations)
                .values(OrganizationInsertion { name: body.0.name })
                .returning(id)
                .get_result::<i32>(&conn)?;
            diesel::insert_into(user_org::users_organizations)
                .values(UsersOrganizationInsertion {
                    user_id: uid,
                    organization_id: org_id,
                })
                .execute(&conn)?;
            Ok(())
        })?;
        return Ok(HttpResponse::build(StatusCode::OK).finish());
    }
    Ok(HttpResponse::build(StatusCode::FORBIDDEN).finish())
}

#[derive(Debug, Clone, Deserialize)]
pub struct VoteCreation {
    name: String,
    deadline: Option<NaiveDate>,
    organization_id: i32,
}

pub async fn create_vote(user_info: UserInfo, body: Json<VoteCreation>, db: DB) -> Result<HttpResponse, Error> {
    use crate::schema::organizations::dsl as org;
    use crate::schema::users_organizations::dsl as user_org;
    use crate::schema::votes::dsl as votes;

    let conn = db.get()?;
    conn.transaction::<_, Error, _>(|| {
        let exists = select(exists(
            user_org::users_organizations
                .filter(user_org::user_id.eq(user_info.id))
                .inner_join(org::organizations)
                .filter(org::id.eq(body.organization_id)),
        ))
        .get_result::<bool>(&conn)?;
        if !exists {
            return Err(Error::BusinessError("irrelative organization".into()));
        }
        crate::diesel::insert_into(votes::votes)
            .values(VoteInsertion {
                name: body.0.name,
                deadline: body.0.deadline,
                status: VoteStatus::Collecting,
                organization_id: body.0.organization_id,
            })
            .execute(&conn)?;

        Ok(())
    })?;
    return Ok(HttpResponse::build(StatusCode::OK).finish());
}

#[derive(Debug, Clone, Deserialize)]
pub struct Opt {
    pub option: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuestionAddition {
    pub description: String,
    pub opts: Vec<Opt>,
}

pub async fn add_question(user_info: UserInfo, Path((org_id, vote_id)): Path<(i32, i32)>, body: Json<QuestionAddition>, db: DB) -> Result<HttpResponse, Error> {
    use crate::schema::options::dsl as opts;
    use crate::schema::questions::dsl as quest;
    use crate::schema::users_organizations::dsl as user_orgs;
    use crate::schema::votes::dsl as votes;
    let conn = db.get()?;
    conn.transaction::<_, Error, _>(|| {
        let exists = select(exists(
            user_orgs::users_organizations
                .inner_join(crate::schema::organizations::table.inner_join(crate::schema::votes::table))
                .filter(user_orgs::user_id.eq(user_info.id).and(user_orgs::organization_id.eq(org_id)).and(votes::id.eq(vote_id))),
        ))
        .get_result::<bool>(&conn)?;
        if !exists {
            return Err(Error::BusinessError("irrelative vote or vote not exists".into()));
        }
        let qid = diesel::insert_into(quest::questions)
            .values(QuestionInsertion {
                description: body.0.description,
                vote_id: vote_id,
            })
            .returning(quest::id)
            .get_result::<i32>(&conn)?;
        let opts: Vec<OptInsertion> = body.0.opts.into_iter().map(|v| OptInsertion { question_id: qid, option: v.option }).collect();
        diesel::insert_into(opts::options).values(opts).execute(&conn)?;
        Ok(())
    })?;
    return Ok(HttpResponse::build(StatusCode::OK).finish());
}

pub async fn add_opts(user_info: UserInfo, Path((org_id, vote_id, qst_id)): Path<(i32, i32, i32)>, body: Json<Vec<Opt>>, db: DB) -> Result<HttpResponse, Error> {
    let conn = db.get()?;
    conn.transaction::<(), Error, _>(|| {
        let question_exists: bool = diesel::select(exists(
            users::table
                .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table.inner_join(questions::table))))
                .filter(users::id.eq(user_info.id).and(organizations::id.eq(org_id)).and(votes::id.eq(vote_id)).and(questions::id.eq(qst_id))),
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

pub async fn submit_answer(user_info: UserInfo, path: Path<(i32, i32)>, db: DB) -> Result<HttpResponse, Error> {
    let conn = db.get()?;
    conn.transaction::<_, Error, _>(|| {
        let is_vote_valid = select(exists(
            options::table
                .inner_join(questions::table.inner_join(votes::table.inner_join(organizations::table.inner_join(users_organizations::table))))
                .filter(
                    options::dsl::id
                        .eq(path.0 .1)
                        .and(questions::dsl::id.eq(path.0 .0))
                        .and(votes::dsl::status.eq(VoteStatus::Collecting))
                        .and(users_organizations::dsl::user_id.eq(user_info.id)),
                ),
        ))
        .get_result::<bool>(&conn)?;
        if !is_vote_valid {
            return Err(Error::BusinessError("vote not exists or invalid vote status".into()));
        }
        diesel::insert_into(answers::table)
            .values(AnswerInsertion {
                user_id: user_info.id,
                option_id: path.0 .1,
            })
            .on_conflict((answers::dsl::option_id, answers::dsl::user_id))
            .do_update()
            .set(answers::dsl::option_id.eq(path.0 .1))
            .execute(&conn)?;
        Ok(())
    })?;
    Ok(HttpResponse::build(StatusCode::FORBIDDEN).finish())
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatesSubmission {
    pub dates: Vec<NaiveDate>,
}

pub async fn submit_dates(user_info: UserInfo, path: Path<(i32,)>, body: Json<DatesSubmission>, db: DB) -> Result<HttpResponse, Error> {
    let conn = db.get()?;
    conn.transaction::<_, Error, _>(|| {
        let is_vote_valid = select(exists(
            users_organizations::table.inner_join(organizations::table.inner_join(votes::table)).filter(
                users_organizations::dsl::user_id
                    .eq(user_info.id)
                    .and(votes::dsl::id.eq(path.0 .0))
                    .and(votes::dsl::status.eq(VoteStatus::Collecting)),
            ),
        ))
        .get_result::<bool>(&conn)?;
        if !is_vote_valid {
            return Err(Error::BusinessError("invalid vote".to_owned()));
        }
        let insertions: Vec<DateInsertion> = body
            .0
            .dates
            .into_iter()
            .map(|v| DateInsertion {
                d: v,
                user_id: user_info.id,
                vote_id: path.0 .0,
            })
            .collect();
        diesel::insert_into(dates::table).values(insertions).execute(&conn)?;
        Ok(())
    })?;
    Ok(HttpResponse::build(StatusCode::OK).finish())
}

#[derive(Debug, Clone, Serialize)]
struct DateReport {
    date: NaiveDate,
    percentage: i32,
}

fn gen_date_report(vote_id: i32, db: DB) -> Result<Vec<DateReport>, Error> {
    let conn = db.get()?;
    let l = dates::table
        .group_by(dates::dsl::d)
        .select((dates::dsl::d, sql::<sql_types::Integer>("(count(*)::float / select count(u.id) from users as u join users_organizations as uo on u.id = uo.user_id join organizations as o on uo.organization_id = o.id join votes as v on o.id = v.organization_id where vote_id = ?) * 100)::int").bind::<sql_types::Integer, _>(vote_id)))
        .load::<(NaiveDate, i32)>(&conn)?;
    Ok(l.into_iter().map(|v| DateReport { date: v.0, percentage: v.1 }).collect())
}

#[derive(Debug, Clone, Serialize)]
struct QuestionReport {
    question: String,
    options: Vec<OptionReport>,
}

use crate::diesel::sql_types::{Integer, Text};

#[derive(Debug, Clone, Serialize, QueryableByName)]
struct OptionReport {
    #[sql_type = "Text"]
    option: String,
    #[sql_type = "Integer"]
    percentage: i32,
}

fn gen_question_report(question_id: i32, db: DB) -> Result<QuestionReport, Error> {
    let conn = db.get()?;
    let question = questions::table.find(question_id).get_result::<Question>(&conn)?;
    let stmt = r#"
    select o.option as option, (count(distinct a.id)::float / (count(distinct uo.user_id)))::int
    from answers as a
    join options as o on a.option_id = o.id
    join questions as q on o.question_id = q.id
    join votes as v on q.vote_id = v.id
    join organizations as oz on v.organization_id = oz.id
    join users_organizations as uo on oz.id = uo.organization_id
    where q.id = $1
    group by option"#;
    let opts = sql_query(stmt).bind::<sql_types::Integer, _>(question_id).load::<OptionReport>(&conn)?;
    Ok(QuestionReport {
        question: question.description,
        options: opts,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrgParam {
    page: i64,
    size: i64,
}

#[derive(Debug, Serialize, Queryable)]
pub struct OrganizationItem {
    id: i32,
    name: String,
    vote_count: i64,
}

pub async fn organization_list(user_info: UserInfo, page: Query<OrgParam>, db: DB) -> Result<HttpResponse, Error> {
    use crate::diesel::sql_types::BigInt;
    use crate::response::List;
    use crate::schema::organizations::dsl as orgs_dsl;
    use crate::schema::users::dsl as users_dsl;
    use crate::schema::users_organizations::dsl as user_orgs_dsl;
    let conn = db.get()?;
    let (orgs, total) = conn.transaction::<(Vec<OrganizationItem>, i64), Error, _>(|| {
        let total = users_dsl::users
            .inner_join(user_orgs_dsl::users_organizations.inner_join(orgs_dsl::organizations))
            .filter(users_dsl::id.eq(user_info.id))
            .count()
            .get_result(&conn)?;
        let orgs = users_dsl::users
            .inner_join(user_orgs_dsl::users_organizations.inner_join(orgs_dsl::organizations.left_join(votes::table)))
            .filter(users_dsl::id.eq(user_info.id))
            .select((organizations::id, organizations::name, crate::diesel::dsl::sql::<BigInt>("count(votes.id) as vote_count")))
            .group_by((organizations::id, organizations::name))
            .limit(page.0.size)
            .offset((page.0.page - 1) * page.0.size)
            .load::<OrganizationItem>(&conn)?;
        Ok((orgs, total))
    })?;
    return Ok(HttpResponse::build(StatusCode::OK).json(List::new(orgs, total)));
}

#[derive(Debug, Serialize, Queryable)]
pub struct OrganizationDetail {
    id: i32,
    name: String,
    votes: Vec<Vote>,
}

pub async fn organization_detail(user_info: UserInfo, Path((organization_id,)): Path<(i32,)>, db: DB) -> Result<HttpResponse, Error> {
    let org = users::table
        .inner_join(users_organizations::table.inner_join(organizations::table))
        .select(organizations::all_columns)
        .filter(users::dsl::id.eq(user_info.id).and(organizations::dsl::id.eq(organization_id)))
        .get_result::<Organization>(&db.get()?)?;
    let votes = Vote::belonging_to(&org).load::<Vote>(&db.get()?)?;
    return Ok(HttpResponse::build(StatusCode::OK).json(OrganizationDetail {
        id: org.id,
        name: org.name,
        votes: votes,
    }));
}

#[derive(Debug, Deserialize)]
pub struct VoteParam {
    page: i64,
    size: i64,
}

pub async fn delete_organization(user_info: UserInfo, Path((organization_id,)): Path<(i32,)>, db: DB) -> Result<HttpResponse, Error> {
    let query = diesel::delete(organizations::table).filter(
        organizations::dsl::id.eq(any(users_organizations::table
            .filter(users_organizations::dsl::user_id.eq(user_info.id).and(users_organizations::dsl::organization_id.eq(organization_id)))
            .select(users_organizations::organization_id))),
    );
    query.execute(&db.get()?)?;
    return Ok(HttpResponse::build(StatusCode::OK).finish());
}

#[derive(Debug, Serialize)]
struct VoteDetail {
    vote: Vote,
    dates: Vec<Date>,
    questions: Vec<Question>,
}

pub async fn vote_list(user_info: UserInfo, param: Query<VoteParam>, organization_id: Path<(i32,)>, db: DB) -> Result<HttpResponse, Error> {
    use crate::response::List;
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
    Ok(HttpResponse::new(StatusCode::FORBIDDEN))
}

pub async fn vote_detail(user_info: UserInfo, Path((org_id, vote_id)): Path<(i32, i32)>, db: DB) -> Result<HttpResponse, Error> {
    let conn = db.get()?;
    let detail = conn.transaction::<VoteDetail, Error, _>(|| {
        let vote: Vote = users::table
            .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table)))
            .filter(users::dsl::id.eq(user_info.id).and(organizations::dsl::id.eq(org_id)).and(votes::dsl::id.eq(vote_id)))
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

#[cfg(test)]
mod test {
    #[test]
    fn test_gen_question_report() {
        use crate::actix_web::web::Data;
        use crate::diesel;
        use crate::diesel::pg::PgConnection;
        use crate::r2d2::Pool;
        dotenv::dotenv().ok();
        let manager = diesel::r2d2::ConnectionManager::<PgConnection>::new("postgres://postgres:postgres@localhost/juju");
        let pool = Pool::new(manager).unwrap();
        super::gen_question_report(1, Data::new(pool)).unwrap();
    }
}
