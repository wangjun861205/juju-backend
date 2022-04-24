use std::fs;

use crate::handlers::upload::FileStorer;
use crate::sha2::{Digest, Sha256};
use std::io::Write;
use std::path::Path;
use std::str;

use crate::error::Error;
struct LocalStorer {
    path: String,
}

impl LocalStorer {
    fn new(path: &str) -> Self {
        Self { path: path.to_owned() }
    }
}

impl FileStorer for LocalStorer {
    fn write(&mut self, bytes: Vec<u8>) -> Result<String, Error> {
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let name = format!("{:x}", hasher.finalize());
        let mut file = fs::File::create(Path::new(&self.path).join(&name))?;
        file.write(&bytes)?;
        Ok(name)
    }
    fn read(&self, id: &str) -> Result<Vec<u8>, Error> {
        Ok(vec![])
    }
}
