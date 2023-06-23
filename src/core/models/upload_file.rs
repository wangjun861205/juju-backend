use serde::Deserialize;

pub struct UploadedFile<I> {
    pub id: I,
    pub name: String,
    pub extension: String,
    pub content: Vec<u8>,
    pub owner_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct UploadedFileCreate {
    pub name: String,
    pub extension: String,
    pub content: Vec<u8>,
    pub owner_id: i32,
}
