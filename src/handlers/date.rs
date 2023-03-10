use std::ops::Bound;

use sqlx::postgres::types::PgRange;
use sqlx::QueryBuilder;

use crate::actix_web::web::{Json, Path, Query};
use crate::chrono::NaiveDate;
use crate::context::UserInfo;
use crate::error::Error;
use crate::handlers::DB;
use crate::models;
use crate::models::DateRangeInsertion;
use crate::serde::{Deserialize, Serialize};
use crate::sqlx::{query, query_as};

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

pub async fn submit_date_ranges(user_info: UserInfo, vote_id: Path<(i32,)>, Json(mut dates): Json<Vec<DateRange>>, db: DB) -> Result<Json<Vec<DateRange>>, Error> {
    dates = merge_date_range(dates);
    let vote_id = vote_id.into_inner().0;
    let mut tx = db.begin()?;
    let (is_valid,): (bool,) = query_as(
        r#"
    SELECT EXISTS(
        SELECT * 
        FROM users AS u
        JOIN users_organizations AS uo ON u.id = uo.user_id
        JOIN organizations AS o ON uo.organization_id = o.id
        JOIN votes AS v ON o.id = votes.organization_id
        WHERE u.id = $1 AND v.id = $2)"#,
    )
    .bind(user_info.id)
    .bind(vote_id)
    .fetch_one(&mut tx)
    .await?;
    if !is_valid {
        return Err(Error::BusinessError("Vote not exists or permission danied".into()));
    }
    query("DELETE date_ranges WHERE user_id = $1 AND vote_id = $2")
        .bind(user_info.id)
        .bind(vote_id)
        .execute(&mut tx)
        .await?;
    QueryBuilder::new("INSERT INTO date_ranges (range, vote_id, user_id)")
        .push_values(dates.iter(), |mut b, d| {
            b.push_bind(PgRange {
                start: Bound::Included(d.start),
                end: Bound::Included(d.end),
            });
            b.push_bind(vote_id);
            b.push_bind(user_info.id);
        })
        .build()
        .execute(&mut tx)
        .await?;
    query("DELETE dates WHERE vote_id = $1 AND user_id = $2").bind(vote_id).bind(user_info.id).execute(&mut tx).await?;
    QueryBuilder::new("INSERT INTO dates (date, user_id, vote_id)")
        .push_tuples(dates.iter(), |mut b, d| {
            b.push_bind(PgRange {
                start: Bound::Included(d.start),
                end: Bound::Included(d.end),
            });
            b.push_bind(user_info.id);
            b.push_bind(vote_id);
        })
        .build()
        .execute(&mut tx)
        .await?;
    Ok(Json(dates))
}

pub async fn date_range_list(user_info: UserInfo, vote_id: Path<(i32,)>, db: DB) -> Result<Json<Vec<DateRange>>, Error> {
    let vote_id = vote_id.into_inner().0;
    let ranges: Vec<models::DateRange> = query_as(
        r#"
    SELECT dr.* 
    FROM users AS u
    JOIN users_organizations AS uo ON u.id = uo.user_id
    JOIN organizations AS o ON uo.organization_id = o.id
    JOIN votes AS v ON o.id = v.organization_id
    JOIN date_ranges AS dr ON v.id = dr.vote_id
    WHERE u.id = $1 AND v.id = $2 AND dr.user_id = $1"#,
    )
    .bind(user_info.id)
    .bind(vote_id)
    .fetch_all(&mut db.acquire().await?)
    .await?;

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

pub async fn month_report(user_info: UserInfo, vote_id: Path<(i32,)>, Query(param): Query<MonthReportParam>, db: DB) -> Result<Json<Vec<MonthReportItem>>, Error> {
    let vote_id = vote_id.into_inner().0;
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

pub async fn year_report(user_info: UserInfo, vote_id: Path<(i32,)>, Query(param): Query<YearReportParam>, db: DB) -> Result<Json<Vec<YearReportItem>>, Error> {
    let vote_id = vote_id.into_inner().0;
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
