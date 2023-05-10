use crate::core::db::{Storer, TxStorer, VoteCommon};
use crate::core::models::VoteCreate;
use crate::models::option::OptInsert;
use crate::models::question::QuestionInsertion;
use crate::models::vote::Vote;
use crate::{error::Error, models::vote::VoteInsertion};

use super::db::{OptionCommon, QuestionCommon};
use super::models::VoteQuery;
use super::uploader::Uploader;

pub async fn create_vote<T, U>(mut storer: T, mut uploader: U, vote: VoteCreate) -> Result<i32, Error>
where
    T: TxStorer,
    U: Uploader<ID = i32>,
{
    let vote_id = VoteCommon::insert(
        &mut storer,
        VoteInsertion {
            name: vote.name,
            deadline: vote.deadline,
            visibility: vote.visibility,
            organization_id: vote.organization_id,
        },
    )
    .await?;
    for q in vote.questions {
        let qst_id = QuestionCommon::insert(
            &mut storer,
            QuestionInsertion {
                description: q.description,
                type_: q.type_,
                version: 1,
                vote_id: vote_id,
            },
        )
        .await?;
        for opt in q.options {
            let img_ids = uploader.bulk_put(opt.images).await?;
            OptionCommon::insert(
                &mut storer,
                OptInsert {
                    option: opt.option,
                    images: img_ids,
                    question_id: qst_id,
                },
            )
            .await?;
        }
    }
    uploader.commit().await?;
    storer.commit().await?;
    Ok(vote_id)
}

pub async fn query_votes<D>(db: &mut D, query: VoteQuery) -> Result<(Vec<Vote>, i64), Error>
where
    D: Storer,
{
    let total = VoteCommon::count(db, &query).await?;
    let votes = VoteCommon::query(db, &query).await?;
    Ok((votes, total))
}
