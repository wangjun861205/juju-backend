use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct OptCreate {
    pub option: String,
    pub images: Vec<String>,
}
