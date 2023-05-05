use crate::error::Error;
pub trait Uploader {
    type ID;
    async fn put(&mut self, data: Vec<u8>) -> Result<Self::ID, Error>;
    async fn get(&mut self, id: Self::ID) -> Result<Vec<u8>, Error>;
    async fn delete(&mut self, id: Self::ID) -> Result<(), Error>;
}
