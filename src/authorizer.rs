use crate::error::Error;
use crate::handlers::authorizer::Authorizer;
use crate::sqlx::{query_as, PgPool};

pub struct PgAuthorizer {
    pool: PgPool,
}

impl PgAuthorizer {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl Authorizer for PgAuthorizer {
    async fn check_organization_read(&self, uid: i32, org_id: i32) -> Result<bool, Error> {
        let mut conn = self.pool.acquire().await?;
        let (is_exists,): (bool,) = query_as("SELECT EXISTS(SELECT * FROM users_organizations WHERE user_id = $1 AND organization_id = $2) AS exists")
            .bind(uid)
            .bind(org_id)
            .fetch_one(&mut conn)
            .await?;
        Ok(is_exists)
    }
    async fn check_organization_write(&self, uid: i32, org_id: i32) -> Result<bool, Error> {
        let mut conn = self.pool.acquire().await?;
        let (is_exists,): (bool,) = query_as("SELECT EXISTS(SELECT * FROM users_organizations WHERE user_id = $1 AND organization_id = $2) AS exists")
            .bind(uid)
            .bind(org_id)
            .fetch_one(&mut conn)
            .await?;
        Ok(is_exists)
    }
    async fn check_vote_read(&self, uid: i32, vote_id: i32) -> Result<bool, Error> {
        let mut conn = self.pool.acquire().await?;
        let (is_exists,): (bool,) = query_as(
            r#"SELECT EXISTS(
                    SELECT * 
                    FROM users AS u
                    JOIN users_organizations AS uo ON u.id = uo.user_id
                    JOIN organizations AS o ON uo.organization_id = o.id 
                    JOIN votes AS v on o.id = v.organization_id
                    WHERE u.id = $1 AND v.id = $2
                )"#,
        )
        .bind(uid)
        .bind(vote_id)
        .fetch_one(&mut conn)
        .await?;

        Ok(is_exists)
    }
    async fn check_vote_write(&self, uid: i32, vote_id: i32) -> Result<bool, Error> {
        let mut conn = self.pool.acquire().await?;
        let (is_exists,): (bool,) = query_as(
            r#"SELECT EXISTS(
                    SELECT * 
                    FROM users AS u
                    JOIN users_organizations AS uo ON u.id = uo.user_id
                    JOIN organizations AS o ON uo.organization_id = o.id 
                    JOIN votes AS v on o.id = v.organization_id
                    WHERE u.id = $1 AND v.id = $2
                )"#,
        )
        .bind(uid)
        .bind(vote_id)
        .fetch_one(&mut conn)
        .await?;

        Ok(is_exists)
    }
    async fn check_question_read(&self, uid: i32, qst_id: i32) -> Result<bool, Error> {
        let mut conn = self.pool.acquire().await?;
        let (is_exists,): (bool,) = query_as(
            r#"
            SELECT EXISTS(
                SELECT *
                FROM users AS u
                JOIN users_organizations AS uo ON u.id = uo.user_id
                JOIN organizations AS o ON uo.organization_id = o.id
                JOIN votes AS v ON v.organization_id = o.id
                JOIN questions AS q ON v.question_id = q.id
                WHERE u.id = $1 AND q.id = $2
            )"#,
        )
        .bind(uid)
        .bind(qst_id)
        .fetch_one(&mut conn)
        .await?;
        Ok(is_exists)
    }
    async fn check_question_write(&self, uid: i32, qst_id: i32) -> Result<bool, Error> {
        let mut conn = self.pool.acquire().await?;
        let (is_exists,): (bool,) = query_as(
            r#"
            SELECT EXISTS(
                SELECT *
                FROM users AS u
                JOIN users_organizations AS uo ON u.id = uo.user_id
                JOIN organizations AS o ON uo.organization_id = o.id
                JOIN votes AS v ON v.organization_id = o.id
                JOIN questions AS q ON v.question_id = q.id
                WHERE u.id = $1 AND q.id = $2
            )"#,
        )
        .bind(uid)
        .bind(qst_id)
        .fetch_one(&mut conn)
        .await?;
        Ok(is_exists)
    }
}
