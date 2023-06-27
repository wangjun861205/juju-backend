use actix_web::{
    http::StatusCode,
    web::{Data, Json},
    HttpResponse,
};
use sqlx::{query, PgPool};

use crate::{
    context::UserInfo,
    core::models::application::{ApplicationStatus, JoinApplicationInsert},
    error::Error,
};

pub async fn create_join_application(user_info: UserInfo, Json(data): Json<JoinApplicationInsert>, db: Data<PgPool>) -> Result<HttpResponse, Error> {
    query("INSERT INTO join_applications (user_id, organization_id, status) VALUES ($1, $2, $3)")
        .bind(user_info.id)
        .bind(data.organization_id)
        .bind(ApplicationStatus::Pending)
        .execute(&mut db.acquire().await?)
        .await?;
    Ok(HttpResponse::new(StatusCode::CREATED))
}
