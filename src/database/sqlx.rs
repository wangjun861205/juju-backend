use crate::core::db::{Common, Manager, OptionCommon, OrganizationCommon, QuestionCommon, QuestionReadMarkCommon, Storer, TxStorer, UserCommon, VoteCommon, VoteReadMarkCommon};
use crate::core::models::vote::VoteVisibility;
use crate::database::models::{
    common::Pagination,
    option::{Insert as OptionInsert, Opt, Query as OptionQuery},
    organization::{Insert as OrganizationInsert, Organization, OrganizationWithVoteInfo, Query as OrganizationQuery, Update as OrganizationUpdate},
    question::{Insert as QuestionInsert, Query as QuestionQuery, Question, ReadMarkInsert as QuestionReadMarkInsert, ReadMarkUpdate as QuestionReadMarkUpdate},
    user::{Patch as UserPath, User},
    vote::{Insert as VoteInsert, Query as VoteQuery, ReadMarkInsert as VoteReadMarkInsert, Vote},
};
use crate::error::Error;
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

    async fn insert(&mut self, data: OrganizationInsert) -> Result<i32, Error> {
        let id = query_scalar("INSERT INTO organizations (name, version) VALUES ($1, $2, $3) RETURNING id")
            .bind(data.name)
            .bind(data.version)
            .bind(data.description)
            .fetch_one(&mut self.executor)
            .await?;
        Ok(id)
    }

    async fn update(&mut self, id: i32, data: OrganizationUpdate) -> Result<(), Error> {
        query("UPDATE organizations SET name = $1, version = $2 WHERE id = $3")
            .bind(data.name)
            .bind(data.version)
            .bind(id)
            .execute(&mut self.executor)
            .await?;
        Ok(())
    }

    async fn count(&mut self, param: OrganizationQuery) -> Result<i64, Error> {
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

    async fn query(&mut self, param: OrganizationQuery, page: i64, size: i64) -> Result<Vec<OrganizationWithVoteInfo>, Error> {
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

    async fn get(&mut self, id: i32) -> Result<User, Error> {
        let user = query_as("SELECT * FROM users WHERE id = $1").bind(id).fetch_one(&mut self.executor).await?;
        Ok(user)
    }

    async fn patch(&mut self, id: i32, user: UserPath) -> Result<(), Error> {
        query(
            "UPDATE users SET 
        nickname = COALESCE($1, nickname),
        phone = COALESCE($2, phone),
        email = COALESCE($3, email),
        password = COALESCE($4, password),
        salt = COALESCE($5, salt),
        avatar = COALESCE($6, avatar)
        WHERE id = $7",
        )
        .bind(user.nickname)
        .bind(user.phone)
        .bind(user.email)
        .bind(user.password)
        .bind(user.salt)
        .bind(user.avatar)
        .bind(id)
        .execute(&mut self.executor)
        .await?;
        Ok(())
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
    async fn insert(&mut self, data: VoteInsert) -> Result<i32, Error> {
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
        if let Some(oid) = query.organization_id_eq {
            stmt.push(" AND organization_id = ");
            stmt.push_bind(oid);
        }
        let (n,) = stmt.build_query_as().fetch_one(&mut self.executor).await?;
        Ok(n)
    }

    async fn query(&mut self, query: &VoteQuery, pagination: Option<Pagination>) -> Result<Vec<Vote>, Error> {
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
        if let Some(oid) = query.organization_id_eq {
            stmt.push(" AND v.organization_id = ").push_bind(oid);
        }
        if let Some(page) = pagination {
            stmt.push(page.to_sql_clause());
        }
        let votes = stmt.build_query_as().fetch_all(&mut self.executor).await?;
        Ok(votes)
    }

    async fn get(&mut self, id: i32) -> Result<Vote, Error> {
        let vote = query_as("SELECT * FROM votes WHERE id = $1").bind(id).fetch_one(&mut self.executor).await?;
        Ok(vote)
    }
}

impl Storer for PgSqlx<PoolConnection<Postgres>> {}
impl<'a> Storer for PgSqlx<Transaction<'a, Postgres>> {}
impl Common for PgSqlx<PoolConnection<Postgres>> {}
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
    async fn insert(&mut self, uid: i32, question: QuestionInsert) -> Result<i32, Error> {
        let id = query_scalar("INSERT INTO questions (description, type_, version, vote_id, owner) VALUES ($1, $2, 1, $3, $4) RETURNING id")
            .bind(question.description)
            .bind(question.type_)
            .bind(question.vote_id)
            .bind(uid)
            .fetch_one(&mut self.executor)
            .await?;
        Ok(id)
    }

    async fn query(&mut self, query: QuestionQuery, pagination: Option<Pagination>) -> Result<Vec<Question>, Error> {
        let mut q = QueryBuilder::new("SELECT * FROM questions WHERE 1 = 1");
        if let Some(vote_id) = query.vote_id {
            q.push(" AND vote_id = ").push_bind(vote_id);
        }
        if let Some(page) = pagination {
            q.push(page.to_sql_clause());
        }
        let questions = q.build_query_as().fetch_all(&mut self.executor).await?;
        Ok(questions)
    }

    async fn get(&mut self, uid: i32, id: i32) -> Result<Question, Error> {
        let question = query_as(
            "
        SELECT 
            q.*, 
            COALESCE(qrm.version, 0) < q.version AS has_updated,
            COUNT(a.id) > 0 AS has_answered
        FROM questions AS q
        LEFT JOIN question_read_marks AS qrm ON q.id = qrm.question_id AND qrm.user_id = $1
        LEFT JOIN options AS o ON o.question_id = q.id
        LEFT JOIN answers AS a ON a.option_id = o.id AND a.user_id = $1
        WHERE q.id = $2
        GROUP BY (q.id, has_updated)",
        )
        .bind(uid)
        .bind(id)
        .fetch_one(&mut self.executor)
        .await?;
        Ok(question)
    }

    async fn delete(&mut self, id: i32) -> Result<(), Error> {
        query("DELETE FROM questions WHERE id = $1").bind(id).execute(&mut self.executor).await?;
        Ok(())
    }

    async fn get_organization_id(&mut self, question_id: i32) -> Result<i32, Error> {
        let oid = query_scalar("SELECT o.id FROM organizations AS o JOIN votes AS v ON v.organization_id = o.id JOIN questions AS q ON q.vote_id = v.id WHERE q.id = $1")
            .bind(question_id)
            .fetch_one(&mut self.executor)
            .await?;
        Ok(oid)
    }

    async fn is_owner(&mut self, uid: i32, question_id: i32) -> Result<bool, Error> {
        let is_owner = query_scalar("SELECT EXISTS(SELECT 1 FROM questions WHERE id = $1 AND owner = $2)")
            .bind(question_id)
            .bind(uid)
            .fetch_one(&mut self.executor)
            .await?;
        Ok(is_owner)
    }
}

impl<E> OptionCommon for PgSqlx<E>
where
    for<'e> &'e mut E: Executor<'e, Database = Postgres>,
{
    async fn insert(&mut self, option: OptionInsert) -> Result<i32, Error> {
        let id = query_scalar("INSERT INTO options (option, question_id, images) VALUES ($1, $2, $3) RETURNING id")
            .bind(option.option)
            .bind(option.question_id)
            .bind(option.images)
            .fetch_one(&mut self.executor)
            .await?;
        Ok(id)
    }

    async fn query(&mut self, query: OptionQuery) -> Result<Vec<Opt>, Error> {
        let mut q = QueryBuilder::new("SELECT * FROM options WHERE 1 = 1");
        if let Some(question_id) = query.question_id {
            q.push(" AND question_id = ").push_bind(question_id);
        }
        if let Some(limit) = query.limit {
            q.push(" LIMIT ").push_bind(limit);
        }
        if let Some(offset) = query.offset {
            q.push(" OFFSET ").push_bind(offset);
        }
        let opts = q.build_query_as().fetch_all(&mut self.executor).await?;
        Ok(opts)
    }

    async fn count(&mut self, query: OptionQuery) -> Result<i64, Error> {
        let mut q = QueryBuilder::new("SELECT COUNT(id) FROM options WHERE 1 = 1");
        if let Some(question_id) = query.question_id {
            q.push(" AND question_id = ").push_bind(question_id);
        }
        let (opts,) = q.build_query_as().fetch_one(&mut self.executor).await?;
        Ok(opts)
    }
}

impl<E> VoteReadMarkCommon for PgSqlx<E>
where
    for<'e> &'e mut E: Executor<'e, Database = Postgres>,
{
    async fn insert(&mut self, mark: VoteReadMarkInsert) -> Result<i32, Error> {
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
    async fn insert(&mut self, mark: QuestionReadMarkInsert) -> Result<i32, Error> {
        let id = query_scalar("INSERT INTO question_read_marks (user_id, question_id, version) VALUES ($1, $2, $3) RETURNING id")
            .bind(mark.user_id)
            .bind(mark.question_id)
            .bind(mark.version)
            .fetch_one(&mut self.executor)
            .await?;
        Ok(id)
    }

    async fn update(&mut self, update: QuestionReadMarkUpdate) -> Result<(), Error> {
        query("UPDATE question_read_marks SET version = $1 WHERE user_id = $2 AND question_id = $3")
            .bind(update.version)
            .bind(update.user_id)
            .bind(update.question_id)
            .execute(&mut self.executor)
            .await?;
        Ok(())
    }
}
