use crate::core::db::Storer;
use crate::database::models::option::{Opt, Query as OptionQuery};
use crate::error::Error;
use default;

use super::db::OptionCommon;
pub async fn options_of_question<S>(storer: &mut S, question_id: i32) -> Result<(Vec<Opt>, i64), Error>
where
    S: Storer,
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
