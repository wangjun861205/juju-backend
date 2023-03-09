use crate::error::Error;
use crate::handlers::authorizer::Authorizer;
use crate::sqlx::{query, query_as, PgPool};

pub struct PgAuthorizer {
    pool: PgPool,
}

impl PgAuthorizer {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Authorizer for PgAuthorizer {
    async fn check_organization_read(&self, uid: i32, org_id: i32) -> Result<bool, Error> {
        let conn = self.pool.acquire().await?;
        let is_exists: bool = query!("SELECT EXISTS(SELECT * FROM users_organizations WHERE user_id = $1 AND organization_id = $2)", uid, org_id)
            .execute(conn)
            .await?;
        Ok(is_exists)
    }
    fn check_organization_write(&self, uid: i32, org_id: i32) -> Result<bool, Error> {
        let conn = self.pool.get()?;
        select(exists(
            users_organizations::table.filter(users_organizations::user_id.eq(uid).and(users_organizations::organization_id.eq(org_id))),
        ))
        .get_result(&conn)
        .map_err(|e| Error::from(e))
    }
    fn check_vote_read(&self, uid: i32, vote_id: i32) -> Result<bool, Error> {
        let conn = self.pool.get()?;
        select(exists(
            users::table
                .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table)))
                .filter(users::id.eq(uid).and(votes::id.eq(vote_id))),
        ))
        .get_result(&conn)
        .map_err(|e| Error::from(e))
    }
    fn check_vote_write(&self, uid: i32, vote_id: i32) -> Result<bool, Error> {
        let conn = self.pool.get()?;
        select(exists(
            users::table
                .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table)))
                .filter(users::id.eq(uid).and(votes::id.eq(vote_id))),
        ))
        .get_result(&conn)
        .map_err(|e| Error::from(e))
    }
    fn check_question_read(&self, uid: i32, qst_id: i32) -> Result<bool, Error> {
        let conn = self.pool.get()?;
        select(exists(
            users::table
                .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table.inner_join(questions::table))))
                .filter(users::id.eq(uid).and(questions::id.eq(qst_id))),
        ))
        .get_result(&conn)
        .map_err(|e| Error::from(e))
    }
    fn check_question_write(&self, uid: i32, qst_id: i32) -> Result<bool, Error> {
        let conn = self.pool.get()?;
        select(exists(
            users::table
                .inner_join(users_organizations::table.inner_join(organizations::table.inner_join(votes::table.inner_join(questions::table))))
                .filter(users::id.eq(uid).and(questions::id.eq(qst_id))),
        ))
        .get_result(&conn)
        .map_err(|e| Error::from(e))
    }
}
