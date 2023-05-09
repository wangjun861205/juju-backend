use std::{
    fs::{read, remove_file, write},
    path::Path,
};

use crate::core::models::{UploadedFile, UploadedFileCreate};
use crate::core::uploader::Uploader;
use crate::error::Error;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct UploadedFileInfo<I> {
    id: I,
    name: String,
    extension: String,
    owner_id: i32,
}

pub struct UploadedFileInfoInsert {
    pub name: String,
    pub extension: String,
    pub owner_id: i32,
}

pub trait InfoStore {
    type ID: Clone;
    async fn insert(&mut self, info: UploadedFileInfoInsert) -> Result<Self::ID, Error>;
    async fn get(&mut self, id: Self::ID) -> Result<UploadedFileInfo<Self::ID>, Error>;
    async fn delete(&mut self, id: Self::ID) -> Result<(), Error>;
}

pub struct LocalStorage<S: InfoStore> {
    path: String,
    store: S,
}

impl<S: InfoStore> LocalStorage<S> {
    pub fn new(path: String, store: S) -> Self {
        Self { path, store }
    }
}

impl<S: InfoStore> Uploader for LocalStorage<S> {
    type ID = S::ID;
    async fn put(&mut self, file: UploadedFileCreate) -> Result<Self::ID, Error> {
        let filename = format!("{}.{}", Uuid::new_v4().to_string(), file.extension);
        write(Path::new(&self.path).join(&filename), file.content)?;
        let id = self
            .store
            .insert(UploadedFileInfoInsert {
                name: filename,
                extension: file.extension,
                owner_id: file.owner_id,
            })
            .await?;
        Ok(id)
    }

    async fn get(&mut self, id: Self::ID) -> Result<UploadedFile<Self::ID>, Error> {
        let info = self.store.get(id.clone()).await?;
        let content = read(Path::new(&self.path).join(&info.name))?;
        Ok(UploadedFile {
            id,
            name: info.name,
            extension: info.extension,
            content,
            owner_id: info.owner_id,
        })
    }

    async fn delete(&mut self, id: Self::ID) -> Result<(), Error> {
        let info = self.store.get(id.clone()).await?;
        remove_file(Path::new(&self.path).join(info.name))?;
        self.store.delete(info.id).await?;
        Ok(())
    }
}
