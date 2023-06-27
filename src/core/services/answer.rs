use crate::core::models::answer::{BulkSubmit as AnswerBulkSubmit, Insert as AnswerInsert, Query as AnswerQuery, Submit as AnswerSubmit};
use crate::core::ports::repository::{AnswerCommon, OptionCommon, QuestionCommon, TxStore};
use crate::error::Error;

pub async fn submit<S>(store: &mut S, user_id: i32, submit: AnswerSubmit) -> Result<(), Error>
where
    S: TxStore,
{
    let is_valid = OptionCommon::is_belongs_to_question(store, submit.question_id, submit.option_ids.clone()).await?;
    if !is_valid {
        return Err(Error::BusinessError("options not belongs to exactly one question".into()));
    }
    AnswerCommon::delete(
        store,
        AnswerQuery {
            question_id_eq: Some(submit.question_id),
            user_id_eq: Some(user_id),
        },
    )
    .await?;
    AnswerCommon::bulk_insert(store, submit.option_ids.into_iter().map(|oid| AnswerInsert { user_id: user_id, option_id: oid }).collect()).await?;
    Ok(())
}

pub async fn bulk_submit<S>(mut store: S, user_id: i32, bulk_submit: AnswerBulkSubmit) -> Result<(), Error>
where
    S: TxStore,
{
    let question_ids = bulk_submit.submissions.iter().map(|s| s.question_id).collect();
    if !QuestionCommon::is_belongs_to_vote(&mut store, bulk_submit.vote_id, question_ids).await? {
        return Err(Error::BusinessError("questions not belongs to exactly one vote".into()));
    }
    for s in bulk_submit.submissions {
        submit(&mut store, user_id, s).await?;
    }
    store.commit().await?;
    Ok(())
}
