use crate::core::models::option::OptCreate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum QuestionType {
    #[default]
    Single,
    Multi,
}

#[derive(Debug, Deserialize)]
pub struct QuestionCreate {
    pub description: String,
    pub type_: QuestionType,
    pub options: Vec<OptCreate>,
}
