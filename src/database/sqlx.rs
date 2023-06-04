use crate::core::{
    db::{Common, Manager, OptionCommon, OrganizationCommon, Pagination, QuestionCommon, QuestionReadMarkCommon, Storer, TxStorer, UserCommon, VoteCommon, VoteReadMarkCommon},
    models::VoteQuery,
};
use crate::error::Error;
use crate::models::option::OptInsert;
use crate::models::{
    organization::{Organization, OrganizationWithVoteInfo, Query},
    question::{Query as QuestionQuery, Question, QuestionInsertion, ReadMarkCreate as QuestionReadMarkCreate},
    user::User,
};
use sqlx::pool::PoolConnection;
use sqlx::{query, query_as, query_scalar, Executor, PgPool, Postgres, QueryBuilder, Transaction};

pub struct PgSqlx<E>
where
    for<'e> &'e mut E: Executor<'e>,
{
    executor: E,
}

impl<E> PgSqlx<E>
where
    for<'e> &'e mut E: Executor<'e>,
{
    pub fn new(executor: E) -> Self {
        Self { executor }
    }
}

impl<E> OrganizationCommon for PgSqlx<E>
where
    for<'e> &'e mut E: Executor<'e, Database = Postgres>,
{
    async fn add_member(&mut self, id: i32, uid: i32) -> Result<(), Error> {
        query("INSERT INTO organization_members (organization_id, user_id) VALUES ($1, $2)")
            .bind(id)
            .bind(uid)
            .execute(&mut self.executor)
            .await?;
        Ok(())
    }

    async fn insert(&mut self, data: crate::models::organization::Insert) -> Result<i32, Error> {
        let id = query_scalar("INSERT INTO organizations (name, version) VALUES ($1, $2, $3) RETURNING id")
            .bind(data.name)
            .bind(data.version)
            .bind(data.description)
            .fetch_one(&mut self.executor)
            .await?;
        Ok(id)
    }

    async fn update(&mut self, id: i32, data: crate::models::organization::Update) -> Result<(), Error> {
        query("UPDATE organizations SET name = $1, version = $2 WHERE id = $3")
            .bind(data.name)
            .bind(data.version)
            .bind(id)
            .execute(&mut self.executor)
            .await?;
        Ok(())
    }

    async fn count(&mut self, param: Query) -> Result<i64, Error> {
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

    async fn query(&mut self, param: Query, page: i64, size: i64) -> Result<Vec<OrganizationWithVoteInfo>, Error> {
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

    async fn delete(&mut self, id: i32) -> Result<(), Error> {
        query("DELETE FROM organizations WHERE id = $1").bind(id).execute(&mut self.executor).await?;
        Ok(())
    }

    async fn exists(&mut self, name: &str) -> Result<bool, Error> {
        let exists = query_scalar("SELECT EXISTS(SELECT * FROM organizations WHERE name = $1)")
            .bind(name)
            .fetch_one(&mut self.executor)
            .await?;
        Ok(exists)
    }

    async fn add_user_version(&mut self, id: i32, uid: i32) -> Result<(), Error> {
        query("INSERT INTO organization_read_marks (organization_id, user_id, version) VALUES ($1, $2, 1)")
            .bind(id)
            .bind(uid)
            .execute(&mut self.executor)
            .await?;
        Ok(())
    }

    async fn update_user_version(&mut self, id: i32, uid: i32, version: i32) -> Result<(), Error> {
        query("UPDATE organization_read_marks SET version = $1 WHERE organization_id = $2 AND user_id = $3")
            .bind(version)
            .bind(id)
            .bind(uid)
            .execute(&mut self.executor)
            .await?;
        Ok(())
    }

    async fn add_manager(&mut self, id: i32, uid: i32) -> Result<(), Error> {
        query("INSERT INTO organization_managers (organization_id, user_id) VALUES ($1, $2)")
            .bind(id)
            .bind(uid)
            .execute(&mut self.executor)
            .await?;
        Ok(())
    }

    async fn get(&mut self, id: i32) -> Result<Organization, Error> {
        let org = query_as("SELECT * FROM organizations WHERE id = $1").bind(id).fetch_one(&mut self.executor).await?;
        Ok(org)
    }

    async fn get_for_update(&mut self, id: i32) -> Result<Organization, Error> {
        let org = query_as("SELECT * FROM organizations WHERE id = $1 FOR UPDATE").bind(id).fetch_one(&mut self.executor).await?;
        Ok(org)
    }

    async fn is_member(&mut self, id: i32, uid: i32) -> Result<bool, Error> {
        let res = query_scalar("SELECT EXISTS(SELECT * FROM organization_members WHERE organization_id = $1 AND user_id = $2)")
            .bind(id)
            .bind(uid)
            .fetch_one(&mut self.executor)
            .await?;
        Ok(res)
    }

    async fn is_manager(&mut self, id: i32, uid: i32) -> Result<bool, Error> {
        let res = query_scalar("SELECT EXISTS(SELECT * FROM organization_managers WHERE organization_id = $1 AND user_id = $2)")
            .bind(id)
            .bind(uid)
            .fetch_one(&mut self.executor)
            .await?;
        Ok(res)
    }
}

impl<E> UserCommon for PgSqlx<E>
where
    for<'e> &'e mut E: Executor<'e, Database = Postgres>,
{
    async fn get_by_phone(&mut self, phone: String) -> Result<Option<User>, Error> {
        let user = query_as("SELECT * FROM users WHERE phone = $1").bind(phone).fetch_optional(&mut self.executor).await?;
        Ok(user)
    }
}

pub struct PgSqlxManager {
    pool: PgPool,
}

impl PgSqlxManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn begin(&self) -> Result<PgSqlx<sqlx::Transaction<Postgres>>, Error> {
        let tx = self.pool.begin().await?;
        Ok(PgSqlx { executor: tx })
    }

    pub async fn acquire(&self) -> Result<PgSqlx<sqlx::pool::PoolConnection<Postgres>>, Error> {
        let conn = self.pool.acquire().await?;
        Ok(PgSqlx { executor: conn })
    }
}

impl<E> VoteCommon for PgSqlx<E>
where
    for<'e> &'e mut E: Executor<'e, Database = Postgres>,
{
    async fn insert(&mut self, data: crate::models::vote::VoteInsertion) -> Result<i32, Error> {
        let id = query_scalar("INSERT INTO votes (name, deadline, organization_id, visibility) VALUES ($1, $2, $3, $4) RETURNING id")
            .bind(data.name)
            .bind(data.deadline)
            .bind(data.organization_id)
            .bind(data.visibility)
            .fetch_one(&mut self.executor)
            .await?;
        Ok(id)
    }

    async fn count(&mut self, query: &VoteQuery) -> Result<i64, Error> {
        let mut stmt = QueryBuilder::new(
            "
        SELECT COUNT(DISTINCT id)
        FROM votes 
        WHERE 1 = 1",
        );
        if let Some(oid) = query.organization_id {
            stmt.push(" AND organization_id = ");
            stmt.push_bind(oid);
        }
        let (n,) = stmt.build_query_as().fetch_one(&mut self.executor).await?;
        Ok(n)
    }

    async fn query(&mut self, query: &VoteQuery) -> Result<Vec<crate::models::vote::Vote>, Error> {
        let mut stmt = QueryBuilder::new(
            "SELECT
            v.*,
            CASE WHEN v.version > COALESCE(vrm.version, 0) THEN true ELSE false END AS has_updated,
            CASE WHEN v.deadline < CURRENT_DATE THEN 'Exprired' ELSE 'Active' END AS status
        FROM votes AS v
        LEFT JOIN vote_read_marks AS vrm ON v.id = vrm.vote_id AND vrm.user_id = ",
        );
        stmt.push_bind(query.uid);
        stmt.push(" WHERE 1 = 1");
        if let Some(oid) = query.organization_id {
            stmt.push(" AND v.organization_id = ").push_bind(oid);
        }
        stmt.push(" LIMIT ").push_bind(query.size);
        stmt.push(" OFFSET ").push_bind((query.page - 1) * query.size);
        let votes = stmt.build_query_as().fetch_all(&mut self.executor).await?;
        Ok(votes)
    }

    async fn get(&mut self, id: i32) -> Result<crate::models::vote::Vote, Error> {
        let vote = query_as("SELECT * FROM votes WHERE id = $1").bind(id).fetch_one(&mut self.executor).await?;
        Ok(vote)
    }
}

impl<'a> Storer for PgSqlx<PoolConnection<Postgres>> {}
impl<'a> Storer for PgSqlx<Transaction<'a, Postgres>> {}
impl<'a> Common for PgSqlx<PoolConnection<Postgres>> {}
impl<'a> Common for PgSqlx<Transaction<'a, Postgres>> {}

impl<'a> TxStorer for PgSqlx<Transaction<'a, Postgres>> {
    async fn commit(self) -> Result<(), Error> {
        self.executor.commit().await?;
        Ok(())
    }

    async fn rollback(self) -> Result<(), Error> {
        self.executor.rollback().await?;
        Ok(())
    }
}

impl<'a> Manager<'a, PgSqlx<PoolConnection<Postgres>>, PgSqlx<Transaction<'a, Postgres>>> for PgSqlxManager {
    async fn db(&'a self) -> Result<PgSqlx<PoolConnection<Postgres>>, Error> {
        let d = self.acquire().await?;
        Ok(d)
    }

    async fn tx(&'a self) -> Result<PgSqlx<Transaction<'a, Postgres>>, Error> {
        let t = self.begin().await?;
        Ok(t)
    }
}

impl<E> QuestionCommon for PgSqlx<E>
where
    for<'e> &'e mut E: Executor<'e, Database = Postgres>,
{
    async fn insert(&mut self, question: QuestionInsertion) -> Result<i32, Error> {
        let id = query_scalar("INSERT INTO questions (description, type_, version, vote_id) VALUES ($1, $2, 1, $3) RETURNING id")
            .bind(question.description)
            .bind(question.type_)
            .bind(question.vote_id)
            .fetch_one(&mut self.executor)
            .await?;
        Ok(id)
    }

    async fn query(&mut self, query: QuestionQuery, pagination: Pagination) -> Result<Vec<Question>, Error> {
        let mut q = QueryBuilder::new("SELECT * FROM questions WHERE 1 = 1");
        if let Some(vote_id) = query.vote_id {
            q.push(" AND vote_id = ").push_bind(vote_id);
        }
        q.push(" LIMIT ").push_bind(pagination.size);
        q.push(" OFFSET ").push_bind((pagination.page - 1) * pagination.size);
        let questions = q.build_query_as().fetch_all(&mut self.executor).await?;
        Ok(questions)
    }
}

impl<E> OptionCommon for PgSqlx<E>
where
    for<'e> &'e mut E: Executor<'e, Database = Postgres>,
{
    async fn insert(&mut self, option: OptInsert) -> Result<i32, Error> {
        let id = query_scalar("INSERT INTO options (option, question_id, images) VALUES ($1, $2, $3) RETURNING id")
            .bind(option.option)
            .bind(option.question_id)
            .bind(option.images)
            .fetch_one(&mut self.executor)
            .await?;
        Ok(id)
    }
}

impl<E> VoteReadMarkCommon for PgSqlx<E>
where
    for<'e> &'e mut E: Executor<'e, Database = Postgres>,
{
    async fn insert(&mut self, mark: crate::models::vote::ReadMarkCreate) -> Result<i32, Error> {
        let id = query_scalar("INSERT INTO vote_read_marks (user_id, vote_id, version) VALUES ($1, $2, $3) RETURNING id")
            .bind(mark.user_id)
            .bind(mark.vote_id)
            .bind(mark.version)
            .fetch_one(&mut self.executor)
            .await?;
        Ok(id)
    }
}

impl<E> QuestionReadMarkCommon for PgSqlx<E>
where
    for<'e> &'e mut E: Executor<'e, Database = Postgres>,
{
    async fn insert(&mut self, mark: QuestionReadMarkCreate) -> Result<i32, Error> {
        let id = query_scalar("INSERT INTO question_read_marks (user_id, question_id, version) VALUES ($1, $2, $3) RETURNING id")
            .bind(mark.user_id)
            .bind(mark.question_id)
            .bind(mark.version)
            .fetch_one(&mut self.executor)
            .await?;
        Ok(id)
    }
}
