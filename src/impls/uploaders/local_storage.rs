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
    async fn bulk_insert(&mut self, infos: Vec<UploadedFileInfoInsert>) -> Result<Vec<Self::ID>, Error>;
    async fn bulk_get(&mut self, ids: Vec<Self::ID>) -> Result<Vec<UploadedFileInfo<Self::ID>>, Error>;
    async fn bulk_delete(&mut self, ids: Vec<Self::ID>) -> Result<(), Error>;
}

pub trait TxInfoStore: InfoStore {
    async fn commit(self) -> Result<(), Error>;
    async fn rollback(self) -> Result<(), Error>;
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

impl<S: TxInfoStore> Uploader for LocalStorage<S> {
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

    async fn bulk_put(&mut self, file: Vec<UploadedFileCreate>) -> Result<Vec<Self::ID>, Error> {
        let uploaded = file
            .into_iter()
            .map(|file| {
                let filename = format!("{}.{}", Uuid::new_v4().to_string(), file.extension);
                write(Path::new(&self.path).join(&filename), file.content)?;
                Result::<_, Error>::Ok(UploadedFileInfoInsert {
                    name: filename,
                    extension: file.extension,
                    owner_id: file.owner_id,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(self.store.bulk_insert(uploaded).await?)
    }

    async fn bulk_get(&mut self, ids: Vec<Self::ID>) -> Result<Vec<UploadedFile<Self::ID>>, Error> {
        let infos = self.store.bulk_get(ids).await?;
        let contents = infos.iter().map(|info| read(Path::new(&self.path).join(&info.name))).collect::<Result<Vec<_>, _>>()?;
        Ok(contents
            .into_iter()
            .zip(infos)
            .map(|(content, info)| UploadedFile {
                id: info.id,
                name: info.name,
                extension: info.extension,
                content,
                owner_id: info.owner_id,
            })
            .collect())
    }

    async fn bulk_delete(&mut self, ids: Vec<Self::ID>) -> Result<(), Error> {
        let infos = self.store.bulk_get(ids.clone()).await?;
        infos
            .into_iter()
            .map(|info| {
                remove_file(Path::new(&self.path).join(info.name))?;
                Result::<_, Error>::Ok(())
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.store.bulk_delete(ids).await?;
        Ok(())
    }

    async fn commit(self) -> Result<(), Error> {
        self.store.commit().await?;
        Ok(())
    }

    async fn rollback(self) -> Result<(), Error> {
        self.store.rollback().await?;
        Ok(())
    }
}
