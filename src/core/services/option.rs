use crate::core::models::option::{Opt, Query as OptionQuery};
use crate::core::ports::repository::{OptionCommon, Store};
use crate::error::Error;
use default;

pub async fn options_of_question<S>(storer: &mut S, question_id: i32) -> Result<(Vec<Opt>, i64), Error>
where
    S: Store,
{
    let total = OptionCommon::count(
        storer,
        OptionQuery {
            question_id: Some(question_id),
            ..default::default()
        },
    )
    .await?;
    let opts = OptionCommon::query(
        storer,
        OptionQuery {
            question_id: Some(question_id),
            ..default::default()
        },
    )
    .await?;
    Ok((opts, total))
}
