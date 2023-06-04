use crate::error::Error;
use crate::models::option::OptInsert;
use crate::models::{
    organization::{Insert as OrganizationInsert, Organization, OrganizationWithVoteInfo, Query as OrganizationQuery, Update as OrganizationUpdate},
    question::{Query as QuestionQuery, Question, QuestionInsertion, ReadMarkCreate as QuestionReadMarkCreate},
    user::User,
    vote::{ReadMarkCreate as VoteReadMarkCreate, Vote, VoteInsertion},
};
use std::future::Future;
use std::pin::Pin;

use super::models::VoteQuery;

pub struct Pagination {
    pub page: i64,
    pub size: i64,
}

pub trait VoteCommon {
    async fn insert(&mut self, data: VoteInsertion) -> Result<i32, Error>;
    async fn query(&mut self, query: &VoteQuery) -> Result<Vec<Vote>, Error>;
    async fn count(&mut self, query: &VoteQuery) -> Result<i64, Error>;
    async fn get(&mut self, id: i32) -> Result<Vote, Error>;
}

pub trait VoteReadMarkCommon {
    async fn insert(&mut self, mark: VoteReadMarkCreate) -> Result<i32, Error>;
}

pub trait OrganizationCommon {
    async fn insert(&mut self, data: OrganizationInsert) -> Result<i32, Error>;
    async fn update(&mut self, id: i32, data: OrganizationUpdate) -> Result<(), Error>;
    async fn query(&mut self, param: OrganizationQuery, page: i64, size: i64) -> Result<Vec<OrganizationWithVoteInfo>, Error>;
    async fn count(&mut self, param: OrganizationQuery) -> Result<i64, Error>;
    async fn delete(&mut self, id: i32) -> Result<(), Error>;
    async fn exists(&mut self, name: &str) -> Result<bool, Error>;
    async fn add_member(&mut self, id: i32, uid: i32) -> Result<(), Error>;
    async fn add_user_version(&mut self, id: i32, uid: i32) -> Result<(), Error>;
    async fn update_user_version(&mut self, id: i32, uid: i32, version: i32) -> Result<(), Error>;
    async fn add_manager(&mut self, id: i32, uid: i32) -> Result<(), Error>;
    async fn get(&mut self, id: i32) -> Result<Organization, Error>;
    async fn get_for_update(&mut self, id: i32) -> Result<Organization, Error>;
    async fn is_member(&mut self, id: i32, uid: i32) -> Result<bool, Error>;
    async fn is_manager(&mut self, id: i32, uid: i32) -> Result<bool, Error>;
}

pub trait QuestionCommon {
    async fn insert(&mut self, question: QuestionInsertion) -> Result<i32, Error>;
    async fn query(&mut self, query: QuestionQuery, pagination: Pagination) -> Result<Vec<Question>, Error>;
}

pub trait QuestionReadMarkCommon {
    async fn insert(&mut self, mark: QuestionReadMarkCreate) -> Result<i32, Error>;
}

pub trait UserCommon {
    async fn get_by_phone(&mut self, phone: String) -> Result<Option<User>, Error>;
}

pub trait OptionCommon {
    async fn insert(&mut self, option: OptInsert) -> Result<i32, Error>;
}

pub trait Common: VoteCommon + OrganizationCommon + UserCommon + QuestionCommon + OptionCommon + VoteReadMarkCommon + QuestionReadMarkCommon {}

pub trait DB: Common {
    type Manager: 'static;
    async fn execute_in_transaction<F, R>(&'static self, f: F) -> Result<R, Error>
    where
        F: FnOnce(Self::Manager) -> Pin<Box<dyn Future<Output = Result<R, Error>>>>;
}

pub trait Storer: Common {}

pub trait TxStorer: Storer {
    async fn commit(self) -> Result<(), Error>;
    async fn rollback(self) -> Result<(), Error>;
}

pub trait Manager<'m, S, T>
where
    S: Storer,
    T: TxStorer,
{
    async fn db(&'m self) -> Result<S, Error>;
    async fn tx(&'m self) -> Result<T, Error>;
}
