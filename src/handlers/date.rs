use std::ops::Bound;

use diesel::sql_types::{BigInt, Date, Integer};

use crate::actix_web::web::{Json, Path, Query};
use crate::chrono::NaiveDate;
use crate::context::UserInfo;
use crate::diesel::{
    delete,
    dsl::{exists, sql},
    insert_into,
    query_dsl::QueryDsl,
    select, sql_query, BoolExpressionMethods, Connection, ExpressionMethods, GroupByDsl, RunQueryDsl,
};
use crate::error::Error;
use crate::handlers::DB;
use crate::models;
use crate::models::DateRangeInsertion;
use crate::schema::*;
use crate::serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

fn merge_date_range(mut ranges: Vec<DateRange>) -> Vec<DateRange> {
    if ranges.is_empty() {
        return Vec::new();
    }
    ranges.sort_by_key(|r| r.start);
    let mut result = vec![ranges[0].clone(); 1];
    for r in ranges.into_iter().skip(1) {
        let prev = result.last_mut().unwrap();
        if prev.end > r.start {
            prev.end = prev.end.max(r.end);
        } else {
            result.push(r);
        }
    }
    result
}

pub async fn submit_date_ranges(user_info: UserInfo, Path((vote_id,)): Path<(i32,)>, Json(mut dates): Json<Vec<DateRange>>, db: DB) -> Result<Json<Vec<DateRange>>, Error> {
    dates = merge_date_range(dates);
    let conn = db.get()?;
    conn.transaction::<(), Error, _>(|| {
        let is_valid: bool = select(exists(
            users::table
                .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table)))
                .filter(users::id.eq(user_info.id).and(votes::id.eq(vote_id))),
        ))
        .get_result(&conn)?;
        if !is_valid {
            return Err(Error::BusinessError("Vote not exists or permission danied".into()));
        }
        delete(date_ranges::table.filter(date_ranges::user_id.eq(user_info.id).and(date_ranges::vote_id.eq(vote_id)))).execute(&conn)?;
        insert_into(date_ranges::table)
            .values(
                dates
                    .iter()
                    .map(|v| DateRangeInsertion {
                        range_: (Bound::Included(v.start), Bound::Included(v.end)),
                        vote_id: vote_id,
                        user_id: user_info.id,
                    })
                    .collect::<Vec<DateRangeInsertion>>(),
            )
            .execute(&conn)?;
        delete(dates::table).filter(dates::vote_id.eq(vote_id).and(dates::user_id.eq(user_info.id))).execute(&conn)?;
        insert_into(dates::table)
            .values(
                dates
                    .iter()
                    .map(|r| {
                        let mut curr = r.start.clone();
                        let mut ds: Vec<models::DateInsertion> = Vec::new();
                        while curr < r.end {
                            ds.push(models::DateInsertion {
                                date_: curr.clone(),
                                user_id: user_info.id,
                                vote_id: vote_id,
                            });
                            curr += chrono::Duration::days(1);
                        }
                        ds
                    })
                    .flatten()
                    .collect::<Vec<models::DateInsertion>>(),
            )
            .execute(&conn)?;
        Ok(())
    })?;
    Ok(Json(dates))
}

pub async fn date_range_list(user_info: UserInfo, Path((vote_id,)): Path<(i32,)>, db: DB) -> Result<Json<Vec<DateRange>>, Error> {
    let ranges = users::table
        .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table.inner_join(date_ranges::table))))
        .filter(users::id.eq(user_info.id).and(votes::id.eq(vote_id)).and(date_ranges::user_id.eq(user_info.id)))
        .select(date_ranges::all_columns)
        .load::<models::DateRange>(&db.get()?)?;
    let res: Vec<DateRange> = ranges
        .into_iter()
        .map(|r| {
            if let Bound::Included(start) = r.range_.0 {
                if let Bound::Excluded(end) = r.range_.1 {
                    return DateRange { start: start, end: end };
                }
            }
            unreachable!()
        })
        .collect();
    Ok(Json(res))
}

const YEAR_STAT: &str = r#"
WITH 
    total AS ( 
        SELECT COUNT(DISTINCT u.id) AS count_
        FROM votes AS v 
        JOIN organizations AS o ON v.organization_id = o.id
        JOIN users_organizations AS uo ON o.id = uo.organization_id
        JOIN users AS u ON uo.user_id = u.id
        WHERE v.id = $1
    ),
    t AS (
        SELECT d.date_ AS date_, COUNT(DISTINCT u.id) AS count_
        FROM votes AS v 
        JOIN organizations AS o ON v.organization_id = o.id
        JOIN users_organizations AS uo ON o.id = uo.organization_id
        JOIN users AS u ON uo.user_id = u.id
        JOIN dates AS d ON v.id = d.vote_id AND u.id = d.user_id
        WHERE  v.id = $1 AND EXTRACT(YEAR FROM d.date_) = $2
        GROUP BY date_
        ORDER BY date_
    ),
    months AS (
        SELECT * FROM (VALUES (1), (2), (3), (4), (5), (6), (7), (8), (9), (10), (11), (12)) AS t (month) 
    )

    SELECT 
        ms.month AS month, 
        SUM(CASE WHEN (t.count_::FLOAT / (SELECT count_ FROM total)::FLOAT) < 0.25 THEN 1 ELSE 0 END) AS u25_count,
        SUM(CASE WHEN (t.count_::FLOAT / (SELECT count_ FROM total)::FLOAT) >= 0.25 AND (t.count_::FLOAT / (SELECT count_ FROM total)::FLOAT) < 0.5 THEN 1 ELSE 0 END) AS u50_count,
        SUM(CASE WHEN (t.count_::FLOAT / (SELECT count_ FROM total)::FLOAT) >= 0.5 AND (t.count_::FLOAT / (SELECT count_ FROM total)::FLOAT) < 0.75 THEN 1 ELSE 0 END) AS u75_count,
        SUM(CASE WHEN (t.count_::FLOAT / (SELECT count_ FROM total)::FLOAT) >= 0.75 AND (t.count_::FLOAT / (SELECT count_ FROM total)::FLOAT) < 1 THEN 1 ELSE 0 END) AS u100_count,
        SUM(CASE WHEN (t.count_::FLOAT / (SELECT count_ FROM total)::FLOAT) = 1 THEN 1 ELSE 0 END) AS p100_count
    FROM 
        months AS ms LEFT JOIN t ON  ms.month = EXTRACT(MONTH FROM t.date_)
    GROUP BY month
    ORDER BY month;
"#;

const MONTH_STAT: &str = r#"
WITH 
    total AS ( 
        SELECT COUNT(DISTINCT u.id) AS count_
        FROM votes AS v 
        JOIN organizations AS o ON v.organization_id = o.id
        JOIN users_organizations AS uo ON o.id = uo.organization_id
        JOIN users AS u ON uo.user_id = u.id
        WHERE v.id = $1
    ),
    t AS (
        SELECT d.date_ AS date_, COUNT(DISTINCT u.id) AS count_
        FROM votes AS v 
        JOIN organizations AS o ON v.organization_id = o.id
        JOIN users_organizations AS uo ON o.id = uo.organization_id
        JOIN users AS u ON uo.user_id = u.id
        JOIN dates AS d ON v.id = d.vote_id AND u.id = d.user_id
        WHERE  v.id = $1 AND EXTRACT(YEAR FROM d.date_) = $2 AND EXTRACT(MONTH FROM d.date_) = $3
        GROUP BY date_
        ORDER BY date_
    ),
    dates (date_) AS (
        SELECT DATE(GENERATE_SERIES(DATE(CONCAT($2, '-01-01')), DATE(CONCAT($2, '-12-31')), '1 day'::interval))
    )
    SELECT ds.date_ AS date_, COALESCE((SUM(t.count_::FLOAT / (SELECT count_ FROM total)::FLOAT) * 10000)::Integer, 0) AS rate
    FROM dates AS ds LEFT JOIN t ON ds.date_ = t.date_
    WHERE EXTRACT(YEAR FROM ds.date_) = $2 AND EXTRACT(MONTH FROM ds.date_) = $3
    GROUP BY ds.date_
    ORDER BY ds.date_
"#;

#[derive(Debug, Deserialize)]
pub struct MonthReportParam {
    pub year: i32,
    pub month: i32,
}

#[derive(Debug, Clone, Serialize, QueryableByName)]
pub struct MonthReportItem {
    #[sql_type = "Date"]
    date_: NaiveDate,
    #[sql_type = "Integer"]
    rate: i32,
}

pub async fn month_report(user_info: UserInfo, Path((vote_id,)): Path<(i32,)>, Query(param): Query<MonthReportParam>, db: DB) -> Result<Json<Vec<MonthReportItem>>, Error> {
    let is_valid: bool = select(exists(
        users::table
            .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table)))
            .filter(users::id.eq(user_info.id).and(votes::id.eq(vote_id))),
    ))
    .get_result(&db.get()?)?;
    if !is_valid {
        return Err(Error::BusinessError("vote does not exists or permission deny".into()));
    }
    let result = sql_query(MONTH_STAT)
        .bind::<Integer, i32>(vote_id)
        .bind::<Integer, i32>(param.year)
        .bind::<Integer, i32>(param.month)
        .get_results::<MonthReportItem>(&db.get()?)?;
    Ok(Json(result))
}

#[derive(Debug, Deserialize)]
pub struct YearReportParam {
    year: i32,
}

#[derive(Debug, Serialize, QueryableByName)]
pub struct YearReportItem {
    #[sql_type = "Integer"]
    month: i32,
    #[sql_type = "BigInt"]
    u25_count: i64,
    #[sql_type = "BigInt"]
    u50_count: i64,
    #[sql_type = "BigInt"]
    u75_count: i64,
    #[sql_type = "BigInt"]
    u100_count: i64,
    #[sql_type = "BigInt"]
    p100_count: i64,
}

pub async fn year_report(user_info: UserInfo, Path((vote_id,)): Path<(i32,)>, Query(param): Query<YearReportParam>, db: DB) -> Result<Json<Vec<YearReportItem>>, Error> {
    let is_valid: bool = select(exists(
        users::table
            .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table)))
            .filter(users::id.eq(user_info.id).and(votes::id.eq(vote_id))),
    ))
    .get_result::<bool>(&db.get()?)?;
    if !is_valid {
        return Err(Error::BusinessError("vote does not exists or permission deny".into()));
    }
    Ok(Json(sql_query(YEAR_STAT).bind::<Integer, _>(vote_id).bind::<Integer, _>(param.year).load(&db.get()?)?))
}

#[derive(Debug, Clone, Serialize)]
struct DateReport {
    date: NaiveDate,
    percentage: i32,
}

fn gen_date_report(vote_id: i32, db: DB) -> Result<Vec<DateReport>, Error> {
    let conn = db.get()?;
    let l = dates::table
        .group_by(dates::dsl::date_)
        .select((dates::dsl::date_, sql::<Integer>("(count(*)::float / select count(u.id) from users as u join users_organizations as uo on u.id = uo.user_id join organizations as o on uo.organization_id = o.id join votes as v on o.id = v.organization_id where vote_id = ?) * 100)::int").bind::<Integer, _>(vote_id)))
        .load::<(NaiveDate, i32)>(&conn)?;
    Ok(l.into_iter().map(|v| DateReport { date: v.0, percentage: v.1 }).collect())
}
