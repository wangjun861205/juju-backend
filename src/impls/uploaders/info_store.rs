use std::ops::{Deref, DerefMut};

use sqlx::{query, query_as, query_scalar, PgConnection, PgExecutor, PgPool, Postgres, QueryBuilder, Transaction};

use crate::error::Error;
use crate::impls::uploaders::local_storage::{InfoStore, TxInfoStore, UploadedFileInfo, UploadedFileInfoInsert};

pub struct SqlxInfoStore<E>
where
    for<'e> &'e mut E: PgExecutor<'e>,
{
    executor: E,
}

impl SqlxInfoStore<PgConnection> {
    pub fn with_conn(executor: PgConnection) -> Self {
        Self { executor }
    }
}

impl<'c> SqlxInfoStore<Transaction<'c, Postgres>> {
    pub fn with_tx(executor: Transaction<'c, Postgres>) -> Self {
        Self { executor }
    }
}

impl<E> InfoStore for SqlxInfoStore<E>
where
    for<'e> &'e mut E: PgExecutor<'e>,
{
    type ID = i32;
    async fn insert(&mut self, info: UploadedFileInfoInsert) -> Result<Self::ID, Error> {
        let id = query_scalar("INSERT INTO uploaded_files (name, extension, owner_id) VALUES ($1, $2, $3) RETURNING id")
            .bind(info.name)
            .bind(info.extension)
            .bind(info.owner_id)
            .fetch_one(&mut self.executor)
            .await?;
        Ok(id)
    }

    async fn get(&mut self, id: Self::ID) -> Result<UploadedFileInfo<Self::ID>, Error> {
        let info = query_as("SELECT * FROM uploaded_files WHERE id = $1").bind(id).fetch_one(&mut self.executor).await?;
        Ok(info)
    }

    async fn delete(&mut self, id: Self::ID) -> Result<(), Error> {
        query("DELETE FROM uploaded_files WHERE id = $1").bind(id).execute(&mut self.executor).await?;
        Ok(())
    }

    async fn bulk_insert(&mut self, infos: Vec<UploadedFileInfoInsert>) -> Result<Vec<Self::ID>, Error> {
        let ids: Vec<(i32,)> = QueryBuilder::new("INSERT INTO uploaded_files (name, extension, owner_id)")
            .push_values(infos, |mut b, info| {
                b.push_bind(info.name).push_bind(info.extension).push_bind(info.owner_id);
            })
            .push("RETURNING id")
            .build_query_as()
            .fetch_all(&mut self.executor)
            .await?;
        Ok(ids.into_iter().map(|(id,)| id).collect())
    }

    async fn bulk_get(&mut self, ids: Vec<Self::ID>) -> Result<Vec<UploadedFileInfo<Self::ID>>, Error> {
        let infos = query_as("SELECT * FROM uploaded_files WHERE id = ANY($1)").bind(ids).fetch_all(&mut self.executor).await?;
        Ok(infos)
    }

    async fn bulk_delete(&mut self, ids: Vec<Self::ID>) -> Result<(), Error> {
        query("DELETE FROM uploaded_files WHERE id = ANY($1)").bind(ids).execute(&mut self.executor).await?;
        Ok(())
    }
}

impl TxInfoStore for SqlxInfoStore<Transaction<'_, Postgres>> {
    async fn commit(self) -> Result<(), Error> {
        self.executor.commit().await?;
        Ok(())
    }

    async fn rollback(self) -> Result<(), Error> {
        self.executor.rollback().await?;
        Ok(())
    }
}

impl TxInfoStore for SqlxInfoStore<PgConnection> {
    async fn commit(self) -> Result<(), Error> {
        Ok(())
    }
    async fn rollback(self) -> Result<(), Error> {
        Ok(())
    }
}
