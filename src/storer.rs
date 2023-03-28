use std::fs::File;

use crate::bytes::Bytes;
use crate::handlers::upload::FileStorer;
use crate::sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::Path;
use std::str;

use crate::error::Error;
pub struct LocalStorer {
    path: String,
}

impl LocalStorer {
    pub fn new(path: &str) -> Self {
        Self { path: path.to_owned() }
    }
}

impl FileStorer for LocalStorer {
    fn write(&self, bytes: Bytes) -> Result<String, Error> {
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let name = format!("{:x}", hasher.finalize());
        let mut file = File::create(Path::new(&self.path).join(&name))?;
        file.write_all(&bytes)?;
        Ok(name)
    }
    fn read(&self, fetch_code: &str) -> Result<Bytes, Error> {
        let mut file = File::open(Path::new(&self.path).join(fetch_code))?;
        let mut content = Vec::new();
        file.read_to_end(&mut content)?;
        Ok(Bytes::from(content))
    }
}
