use crate::diesel::{r2d2::ConnectionManager, PgConnection};
use crate::error::Error;
use crate::handlers::authorizer::Authorizer;
use crate::r2d2::Pool;

type ConnectionPool = Pool<ConnectionManager<PgConnection>>;

pub struct PgAuthorizer {
    pool: ConnectionPool,
}

impl PgAuthorizer {
    pub fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }
}

impl Authorizer for PgAuthorizer {
    fn check_organization_read(&self, uid: i32, org_id: i32) -> Result<bool, Error> {
        Ok(true)
    }
    fn check_organization_write(&self, uid: i32, org_id: i32) -> Result<bool, Error> {
        Ok(true)
    }
    fn check_vote_read(&self, uid: i32, vote_id: i32) -> Result<bool, Error> {
        Ok(true)
    }
    fn check_vote_write(&self, uid: i32, vote_id: i32) -> Result<bool, Error> {
        Ok(true)
    }
    fn check_question_read(&self, uid: i32, question_id: i32) -> Result<bool, Error> {
        Ok(true)
    }
    fn check_question_write(&self, uid: i32, question_id: i32) -> Result<bool, Error> {
        Ok(true)
    }
}
