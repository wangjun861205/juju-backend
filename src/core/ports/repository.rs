use crate::core::models::{
    answer::{Insert as AnswerInsert, Query as AnswerQuery},
    common::Pagination,
    option::{Insert as OptionInsert, Opt, Query as OptionQuery},
    organization::{Insert as OrganizationInsert, Organization, OrganizationWithVoteInfo, Query as OrganizationQuery, Update as OrganizationUpdate},
    question::{Insert as QuestionInsert, Query as QuestionQuery, Question, ReadMarkInsert as QuestionReadMarkInsert, ReadMarkUpdate as QuestionReadMarkUpdate},
    user::{Patch as UserPatch, User},
    vote::{Insert as VoteInsert, Query as VoteQuery, ReadMarkInsert as VoteReadMarkInsert, Vote},
};
use crate::error::Error;
use std::future::Future;
use std::pin::Pin;

pub trait VoteCommon {
    async fn insert(&mut self, data: VoteInsert) -> Result<i32, Error>;
    async fn query(&mut self, query: &VoteQuery, pagination: Option<Pagination>) -> Result<Vec<Vote>, Error>;
    async fn count(&mut self, query: &VoteQuery) -> Result<i64, Error>;
    async fn get(&mut self, uid: i32, id: i32) -> Result<Vote, Error>;
    async fn update_read_mark_version(&mut self, uid: i32, id: i32, version: i64) -> Result<(), Error>;
}

pub trait VoteReadMarkCommon {
    async fn insert(&mut self, mark: VoteReadMarkInsert) -> Result<i32, Error>;
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
    async fn insert(&mut self, uid: i32, question: QuestionInsert) -> Result<i32, Error>;
    async fn query(&mut self, uid: i32, query: QuestionQuery, pagination: Option<Pagination>) -> Result<Vec<Question>, Error>;
    async fn count(&mut self, query: QuestionQuery) -> Result<i64, Error>;
    async fn get(&mut self, uid: i32, id: i32) -> Result<Question, Error>;
    async fn delete(&mut self, id: i32) -> Result<(), Error>;
    async fn get_organization_id(&mut self, question_id: i32) -> Result<i32, Error>;
    async fn is_owner(&mut self, uid: i32, id: i32) -> Result<bool, Error>;
    async fn is_belongs_to_vote(&mut self, vote_id: i32, ids: Vec<i32>) -> Result<bool, Error>;
}

pub trait QuestionReadMarkCommon {
    async fn insert(&mut self, mark: QuestionReadMarkInsert) -> Result<i32, Error>;
    async fn update(&mut self, update: QuestionReadMarkUpdate) -> Result<(), Error>;
}

pub trait UserCommon {
    async fn get_by_phone(&mut self, phone: String) -> Result<Option<User>, Error>;
    async fn get(&mut self, id: i32) -> Result<User, Error>;
    async fn patch(&mut self, id: i32, user: UserPatch) -> Result<(), Error>;
}

pub trait OptionCommon {
    async fn insert(&mut self, option: OptionInsert) -> Result<i32, Error>;
    async fn query(&mut self, query: OptionQuery) -> Result<Vec<Opt>, Error>;
    async fn count(&mut self, query: OptionQuery) -> Result<i64, Error>;
    async fn count_question(&mut self, query: OptionQuery) -> Result<i64, Error>;
    async fn is_belongs_to_question(&mut self, question_id: i32, ids: Vec<i32>) -> Result<bool, Error>;
}

pub trait AnswerCommon {
    async fn insert(&mut self, answer: AnswerInsert) -> Result<i32, Error>;
    async fn bulk_insert(&mut self, answers: Vec<AnswerInsert>) -> Result<(), Error>;
    async fn delete(&mut self, query: AnswerQuery) -> Result<i32, Error>;
}

pub trait Common: VoteCommon + OrganizationCommon + UserCommon + QuestionCommon + OptionCommon + VoteReadMarkCommon + QuestionReadMarkCommon + AnswerCommon {}

pub trait DB: Common {
    type Manager: 'static;
    async fn execute_in_transaction<F, R>(&'static self, f: F) -> Result<R, Error>
    where
        F: FnOnce(Self::Manager) -> Pin<Box<dyn Future<Output = Result<R, Error>>>>;
}

pub trait Store: Common {}

pub trait TxStore: Store {
    async fn commit(self) -> Result<(), Error>;
    async fn rollback(self) -> Result<(), Error>;
}

pub trait Manager<'m, S, T>
where
    S: Store,
    T: TxStore,
{
    async fn db(&'m self) -> Result<S, Error>;
    async fn tx(&'m self) -> Result<T, Error>;
}
