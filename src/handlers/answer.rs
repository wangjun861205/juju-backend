use crate::context::UserInfo;
use crate::core::services::answer::{bulk_submit, submit};
use crate::error::Error;
use crate::sqlx::PgPool;
use crate::{
    actix_web::{
        http::StatusCode,
        web::{Data, Json, Path},
        HttpResponse,
    },
    core::models::answer::{BulkSubmit, Submit},
    core::ports::repository::TxStore,
    database::sqlx::PgSqlx,
};

pub async fn submit_answer(user_info: UserInfo, qst_id: Path<(i32,)>, Json(answer): Json<Vec<i32>>, db: Data<PgPool>) -> Result<HttpResponse, Error> {
    let qst_id = qst_id.into_inner().0;
    let tx = db.begin().await?;
    let mut store = PgSqlx::new(tx);
    submit(
        &mut store,
        user_info.id,
        Submit {
            question_id: qst_id,
            option_ids: answer,
        },
    )
    .await?;
    store.commit().await?;
    Ok(HttpResponse::build(StatusCode::OK).finish())
}

pub async fn submit_answers(db: Data<PgPool>, user_info: UserInfo, Json(answers): Json<BulkSubmit>) -> Result<HttpResponse, Error> {
    let tx = db.begin().await?;
    let store = PgSqlx::new(tx);
    bulk_submit(store, user_info.id, answers).await?;
    Ok(HttpResponse::build(StatusCode::CREATED).finish())
}
