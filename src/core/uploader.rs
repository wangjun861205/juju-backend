use crate::core::models::{UploadedFile, UploadedFileCreate};

use crate::error::Error;
pub trait Uploader {
    type ID;
    async fn put(&mut self, file: UploadedFileCreate) -> Result<Self::ID, Error>;
    async fn get(&mut self, id: Self::ID) -> Result<UploadedFile<Self::ID>, Error>;
    async fn delete(&mut self, id: Self::ID) -> Result<(), Error>;
}
