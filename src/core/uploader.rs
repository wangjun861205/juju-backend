use crate::core::models::{UploadedFile, UploadedFileCreate};

use crate::error::Error;
pub trait Uploader {
    type ID;
    async fn put(&mut self, file: UploadedFileCreate) -> Result<Self::ID, Error>;
    async fn get(&mut self, id: Self::ID) -> Result<UploadedFile<Self::ID>, Error>;
    async fn delete(&mut self, id: Self::ID) -> Result<(), Error>;
    async fn bulk_put(&mut self, file: Vec<UploadedFileCreate>) -> Result<Vec<Self::ID>, Error>;
    async fn bulk_get(&mut self, ids: Vec<Self::ID>) -> Result<Vec<UploadedFile<Self::ID>>, Error>;
    async fn bulk_delete(&mut self, ids: Vec<Self::ID>) -> Result<(), Error>;
    async fn commit(self) -> Result<(), Error>;
    async fn rollback(self) -> Result<(), Error>;
}
