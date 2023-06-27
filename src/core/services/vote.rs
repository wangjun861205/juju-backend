use crate::core::models::{
    common::Pagination,
    option::Insert as OptionInsert,
    question::{Insert as QuestionInsert, QuestionType, ReadMarkInsert as QuestionReadMarkInsert},
    vote::{Insert as VoteInsert, Query as DBVoteQuery, ReadMarkInsert as VoteReadMarkInsert, Vote, VoteCreate, VoteQuery, VoteVisibility},
};
use crate::core::ports::repository::{OptionCommon, QuestionCommon, QuestionReadMarkCommon, Store, TxStore, VoteCommon, VoteReadMarkCommon};
use crate::error::Error;

pub async fn create_vote<T>(mut storer: T, uid: i32, vote: VoteCreate) -> Result<i32, Error>
where
    T: TxStore,
{
    // 创建投票
    let vote_id = VoteCommon::insert(
        &mut storer,
        VoteInsert {
            name: vote.name,
            deadline: vote.deadline,
            visibility: match vote.visibility {
                VoteVisibility::Public => "Public".into(),
                VoteVisibility::Organization => "Organization".into(),
                VoteVisibility::WhiteList => "WhiteList".into(),
            },
            organization_id: vote.organization_id,
        },
    )
    .await?;
    // 创建投票阅读标记
    VoteReadMarkCommon::insert(&mut storer, VoteReadMarkInsert { vote_id, user_id: uid, version: 1 }).await?;
    for q in vote.questions {
        // 创建问题
        let qst_id = QuestionCommon::insert(
            &mut storer,
            uid,
            QuestionInsert {
                description: q.description,
                type_: match q.type_ {
                    QuestionType::Multi => "Multi".into(),
                    QuestionType::Single => "Single".into(),
                },
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
    D: Store,
{
    let total = VoteCommon::count(
        db,
        &DBVoteQuery {
            uid: query.uid,
            organization_id_eq: query.organization_id,
        },
    )
    .await?;
    let votes = VoteCommon::query(
        db,
        &DBVoteQuery {
            uid: query.uid,
            organization_id_eq: query.organization_id,
        },
        Some(Pagination::new(query.size, Some((query.page - 1) * query.size))),
    )
    .await?;
    Ok((votes, total))
}

pub async fn vote_detail<D>(db: &mut D, uid: i32, id: i32) -> Result<Vote, Error>
where
    D: Store,
{
    let vote = VoteCommon::get(db, uid, id).await?;
    VoteCommon::update_read_mark_version(db, uid, id, vote.version).await?;
    Ok(vote)
}