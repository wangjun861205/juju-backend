use crate::{
    core::{
        models::{
            option::Insert as OptionInsert,
            question::{Create as QuestionCreate, Insert as QuestionInsert, Query, Question, ReadMarkInsert as QuestionReadMarkInsert, ReadMarkUpdate as QuestionReadMarkUpdate},
        },
        ports::repository::{OptionCommon, OrganizationCommon, QuestionCommon, QuestionReadMarkCommon, Store},
    },
    error::Error,
};

pub async fn create_question<S>(uid: i32, vote_id: i32, storer: &mut S, question: QuestionCreate) -> Result<i32, Error>
where
    S: Store,
{
    let qid = QuestionCommon::insert(
        storer,
        uid,
        QuestionInsert {
            description: question.description,
            version: question.version,
            type_: question.type_,
            vote_id,
        },
    )
    .await?;
    QuestionReadMarkCommon::insert(
        storer,
        QuestionReadMarkInsert {
            question_id: qid,
            user_id: uid,
            version: question.version,
        },
    )
    .await?;
    for o in question.options {
        OptionCommon::insert(
            storer,
            OptionInsert {
                question_id: qid,
                option: o.option,
                images: o.images,
            },
        )
        .await?;
    }
    Ok(qid)
}

pub async fn question_detail<S>(storer: &mut S, uid: i32, id: i32) -> Result<Question, Error>
where
    S: Store,
{
    let question = QuestionCommon::get(storer, uid, id).await?;
    QuestionReadMarkCommon::update(
        storer,
        QuestionReadMarkUpdate {
            question_id: id,
            user_id: uid,
            version: question.version,
        },
    )
    .await?;
    Ok(question)
}

pub async fn delete_question<S>(storer: &mut S, uid: i32, id: i32) -> Result<(), Error>
where
    S: Store,
{
    let org_id = QuestionCommon::get_organization_id(storer, id).await?;
    if !OrganizationCommon::is_manager(storer, org_id, uid).await? && !QuestionCommon::is_owner(storer, uid, id).await? {
        return Err(Error::Unauthorized);
    }
    QuestionCommon::delete(storer, id).await?;
    Ok(())
}

pub async fn questions_with_in_vote<S>(storer: &mut S, uid: i32, vote_id: i32) -> Result<(Vec<Question>, i64), Error>
where
    S: Store,
{
    let total = QuestionCommon::count(storer, Query { vote_id_eq: Some(vote_id) }).await?;
    let questions = QuestionCommon::query(storer, uid, Query { vote_id_eq: Some(vote_id) }, None).await?;
    Ok((questions, total))
}
