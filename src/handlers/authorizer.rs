use crate::error::Error;

pub trait Authorizer {
    async fn check_organization_read(&self, uid: i32, org_id: i32) -> Result<bool, Error>;
    async fn check_organization_write(&self, uid: i32, org_id: i32) -> Result<bool, Error>;
    async fn check_vote_read(&self, uid: i32, vote_id: i32) -> Result<bool, Error>;
    async fn check_vote_write(&self, uid: i32, vote_id: i32) -> Result<bool, Error>;
    async fn check_question_read(&self, uid: i32, question_id: i32) -> Result<bool, Error>;
    async fn check_question_write(&self, uid: i32, question_id: i32) -> Result<bool, Error>;
}
