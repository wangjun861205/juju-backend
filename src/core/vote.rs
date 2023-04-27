use crate::{error::Error, models::vote::VoteInsertion};
pub trait DatabaseManager {
    fn insert(&mut self, data: VoteInsertion) -> Result<i32, Error>;
}

pub trait TransactionManager<D, R, F>: DatabaseManager
where
    D: DatabaseManager,
{
    fn execute_in_transaction(&mut self, f: impl FnOnce(D) -> Result<R, Error>) -> Result<R, Error>;
}

pub async fn create_vote<D, T, F>(mut manager: T, data: VoteInsertion) -> Result<i32, Error>
where
    D: DatabaseManager,
    T: TransactionManager<D, i32, F>,
    F: FnOnce(D) -> Result<i32, Error>,
{
    manager.execute_in_transaction(|mut manager: D| manager.insert(data))
}
