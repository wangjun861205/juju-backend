use crate::core::db::{Storer, TxStorer, VoteCommon};
use crate::core::models::VoteCreate;
use crate::models::option::OptInsert;
use crate::models::question::{QuestionInsertion, ReadMarkCreate as QuestionReadMarkCreate};
use crate::models::vote::{ReadMarkCreate as VoteReadMarkCreate, Vote};
use crate::{error::Error, models::vote::VoteInsertion};

use super::db::{OptionCommon, QuestionCommon, QuestionReadMarkCommon, VoteReadMarkCommon};
use super::models::VoteQuery;

pub async fn create_vote<T>(mut storer: T, uid: i32, vote: VoteCreate) -> Result<i32, Error>
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
    VoteReadMarkCommon::insert(&mut storer, VoteReadMarkCreate { vote_id, user_id: uid, version: 1 }).await?;
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
        QuestionReadMarkCommon::insert(
            &mut storer,
            QuestionReadMarkCreate {
                question_id: qst_id,
                user_id: uid,
                version: 1,
            },
        )
        .await?;
        for opt in q.options {
            OptionCommon::insert(
                &mut storer,
                OptInsert {
                    option: opt.option,
                    images: opt.images,
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

pub async fn vote_detail<D>(db: &mut D, id: i32) -> Result<Vote, Error>
where
    D: Storer,
{
    let vote = VoteCommon::get(db, id).await?;
    Ok(vote)
}
