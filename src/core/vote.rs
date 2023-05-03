use crate::core::db::{Storer, TxStorer, VoteCommon};
use crate::core::models::VoteCreate;
use crate::models::option::OptInsertion;
use crate::models::question::QuestionInsertion;
use crate::models::vote::Vote;
use crate::{error::Error, models::vote::VoteInsertion};

use super::db::{OptionCommon, QuestionCommon};
use super::models::VoteQuery;

pub async fn create_vote<T>(mut storer: T, vote: VoteCreate) -> Result<i32, Error>
where
    T: TxStorer,
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
            OptionCommon::insert(
                &mut storer,
                OptInsertion {
                    option: opt.option,
                    question_id: qst_id,
                },
            )
            .await?;
        }
    }
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
