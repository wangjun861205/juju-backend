use std::{
    fs::{read, remove_file, write},
    path::Path,
};

use crate::core::uploader::Uploader;
use crate::error::Error;
use uuid::Uuid;

pub struct LocalStorage {
    path: String,
}

impl LocalStorage {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl Uploader for LocalStorage {
    type ID = String;
    async fn put(&mut self, data: Vec<u8>) -> Result<Self::ID, Error> {
        let id = Uuid::new_v4().to_string();
        write(Path::new(&self.path).join(&id), data)?;
        Ok(id)
    }

    async fn get(&mut self, id: Self::ID) -> Result<Vec<u8>, Error> {
        let content = read(Path::new(&self.path).join(id))?;
        Ok(content)
    }

    async fn delete(&mut self, id: Self::ID) -> Result<(), Error> {
        remove_file(Path::new(&self.path).join(id))?;
        Ok(())
    }
}
