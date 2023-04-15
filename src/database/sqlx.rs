use crate::core::organization::DatabaseManager as OrganizationManager;
use crate::error::Error;
use crate::models::organization::{Organization, Query};
use sqlx::{query, query_as, query_scalar, Executor, PgPool, Postgres, Transaction};
pub struct PgSqlxManager<E>
where
    for<'e> &'e mut E: Executor<'e>,
{
    executor: E,
}

impl<E> PgSqlxManager<E>
where
    for<'e> &'e mut E: Executor<'e>,
{
    pub fn new(executor: E) -> Self {
        Self { executor }
    }
}

impl PgSqlxManager<Transaction<'_, Postgres>> {
    pub async fn commit(self) -> Result<(), Error> {
        self.executor.commit().await?;
        Ok(())
    }
}

impl<E> OrganizationManager for PgSqlxManager<E>
where
    for<'e> &'e mut E: Executor<'e, Database = Postgres>,
{
    type Error = Error;
    async fn add_member(&mut self, id: i32, uid: i32) -> Result<(), Self::Error> {
        query("INSERT INTO organization_members (organization_id, user_id) VALUES ($1, $2)")
            .bind(id)
            .bind(uid)
            .execute(&mut self.executor)
            .await?;
        Ok(())
    }

    async fn insert(&mut self, data: crate::models::organization::Insert) -> Result<i32, Self::Error> {
        let id = query_scalar("INSERT INTO organizations (name, version) VALUES ($1, $2) RETURNING id")
            .bind(data.name)
            .bind(data.version)
            .fetch_one(&mut self.executor)
            .await?;
        Ok(id)
    }

    async fn update(&mut self, id: i32, data: crate::models::organization::Update) -> Result<(), Self::Error> {
        query("UPDATE organizations SET name = COALESCE($1, name), version = COALESCE($2, version) WHERE id = $3")
            .bind(data.name)
            .bind(data.version)
            .bind(id)
            .execute(&mut self.executor)
            .await?;
        Ok(())
    }

    async fn count(&mut self, param: Query) -> Result<i64, Self::Error> {
        let total = query_scalar(
            "
        SELECT COUNT(DISTINCT id) 
        FROM organizations
        WHERE ($1 IS NULL OR name = $1) 
            AND ($2 IS NULL OR name LIKE $2)
            AND ($3 IS NULL OR id IN (SELECT organization_id FROM organization_members WHERE user_id = $3))",
        )
        .bind(&param.name_eq)
        .bind(&param.name_like.as_ref().map(|v| format!("'%{}%'", v)))
        .bind(&param.member_id)
        .fetch_one(&mut self.executor)
        .await?;
        Ok(total)
    }

    async fn query(&mut self, param: Query, page: i64, size: i64) -> Result<Vec<Organization>, Self::Error> {
        let organizations = query_as(
            "
        SELECT 
            o.id AS id, 
            o.name AS name,
            o.version AS version,
            o.version > COALESCE(orm.version, 0) AS has_new_vote,
            COUNT(DISTINCT v.id) AS vote_count
        FROM organizations AS o 
        LEFT JOIN organization_read_marks AS orm ON o.id = orm.organization_id AND orm.user_id = $3
        LEFT JOIN votes AS v ON o.id = v.organization_id
        WHERE ($1 IS NULL OR o.name = $1) 
            AND ($2 IS NULL OR o.name LIKE $2) 
            AND ($3 IS NULL OR o.id IN (SELECT organization_id FROM organization_members WHERE user_id = $3))
        GROUP BY o.id, o.name, o.version, has_new_vote
        LIMIT $4 
        OFFSET $5",
        )
        .bind(&param.name_eq)
        .bind(&param.name_like.as_ref().map(|v| format!("'%{}%'", v)))
        .bind(&param.member_id)
        .bind(size)
        .bind((page - 1) * size)
        .fetch_all(&mut self.executor)
        .await?;
        Ok(organizations)
    }

    async fn delete(&mut self, id: i32) -> Result<(), Self::Error> {
        query("DELETE FROM organizations WHERE id = $1").bind(id).execute(&mut self.executor).await?;
        Ok(())
    }

    async fn exists(&mut self, name: &str) -> Result<bool, Self::Error> {
        let exists = query_scalar("SELECT EXISTS(SELECT * FROM organizations WHERE name = $1)")
            .bind(name)
            .fetch_one(&mut self.executor)
            .await?;
        Ok(exists)
    }

    async fn add_user_version(&mut self, id: i32, uid: i32) -> Result<(), Self::Error> {
        query("INSERT INTO organization_read_marks (organization_id, user_id, version) VALUES ($1, $2, 1)")
            .bind(id)
            .bind(uid)
            .execute(&mut self.executor)
            .await?;
        Ok(())
    }

    async fn update_user_version(&mut self, id: i32, uid: i32, version: i32) -> Result<(), Self::Error> {
        query("UPDATE organization_read_marks SET version = $1 WHERE organization_id = $2 AND user_id = $3")
            .bind(version)
            .bind(id)
            .bind(uid)
            .execute(&mut self.executor)
            .await?;
        Ok(())
    }

    async fn add_manager(&mut self, id: i32, uid: i32) -> Result<(), Self::Error> {
        query("INSERT INTO organization_managers (organization_id, user_id) VALUES ($1, $2)")
            .bind(id)
            .bind(uid)
            .execute(&mut self.executor)
            .await?;
        Ok(())
    }
}

pub struct PgSqlx {
    pool: PgPool,
}

impl PgSqlx {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn begin(&'static self) -> Result<PgSqlxManager<sqlx::Transaction<Postgres>>, Error> {
        let tx = self.pool.begin().await?;
        Ok(PgSqlxManager { executor: tx })
    }

    pub async fn acquire(&'static self) -> Result<PgSqlxManager<sqlx::pool::PoolConnection<Postgres>>, Error> {
        let conn = self.pool.acquire().await?;
        Ok(PgSqlxManager { executor: conn })
    }
}
