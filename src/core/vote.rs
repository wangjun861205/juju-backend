use crate::core::db::{Storer, TxStorer, VoteCommon};
use crate::core::models::VoteCreate;
use crate::models::option::Insert as OptionInsert;
use crate::models::question::{Insert as QuestionInsert, ReadMarkInsert as QuestionReadMarkInsert};
use crate::models::vote::{ReadMarkCreate as VoteReadMarkCreate, Vote};
use crate::{error::Error, models::vote::VoteInsertion};

use super::db::{OptionCommon, QuestionCommon, QuestionReadMarkCommon, VoteReadMarkCommon};
use super::models::VoteQuery;

pub async fn create_vote<T>(mut storer: T, uid: i32, vote: VoteCreate) -> Result<i32, Error>
where
    T: TxStorer,
{
    // 创建投票
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
    // 创建投票阅读标记
    VoteReadMarkCommon::insert(&mut storer, VoteReadMarkCreate { vote_id, user_id: uid, version: 1 }).await?;
    for q in vote.questions {
        // 创建问题
        let qst_id = QuestionCommon::insert(
            &mut storer,
            uid,
            QuestionInsert {
                description: q.description,
                type_: q.type_,
                version: 1,
                vote_id,
            },
        )
        .await?;
        // 创建问题阅读标记
        QuestionReadMarkCommon::insert(
            &mut storer,
            QuestionReadMarkInsert {
                question_id: qst_id,
                user_id: uid,
                version: 1,
            },
        )
        .await?;
        for opt in q.options {
            // 创建选项
            OptionCommon::insert(
                &mut storer,
                OptionInsert {
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
