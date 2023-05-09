use sqlx::{query, query_as, query_scalar, PgPool};

use crate::error::Error;
use crate::impls::uploaders::local_storage::{InfoStore, UploadedFileInfo, UploadedFileInfoInsert};

pub struct SqlxInfoStore {
    pool: PgPool,
}

impl SqlxInfoStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl InfoStore for SqlxInfoStore {
    type ID = i32;
    async fn insert(&mut self, info: UploadedFileInfoInsert) -> Result<Self::ID, Error> {
        let id = query_scalar("INSERT INTO uploaded_files (name, extension, owner_id) VALUES ($1, $2, $3) RETURNING id")
            .bind(info.name)
            .bind(info.extension)
            .bind(info.owner_id)
            .fetch_one(&mut self.pool.acquire().await?)
            .await?;
        Ok(id)
    }

    async fn get(&mut self, id: Self::ID) -> Result<UploadedFileInfo<Self::ID>, Error> {
        let info = query_as("SELECT * FROM uploaded_files WHERE id = $1").bind(id).fetch_one(&mut self.pool.acquire().await?).await?;
        Ok(info)
    }

    async fn delete(&mut self, id: Self::ID) -> Result<(), Error> {
        query("DELETE FROM uploaded_files WHERE id = $1").bind(id).execute(&mut self.pool.acquire().await?).await?;
        Ok(())
    }
}
