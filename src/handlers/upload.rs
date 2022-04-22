use actix_web::FromRequest;

use crate::actix_multipart::{Field, Multipart, MultipartError};
use crate::actix_web::web::{Data, Json};
use crate::bytes::Bytes;
use crate::error::Error;
use crate::futures_util::{Stream, StreamExt, TryStream, TryStreamExt};
use crate::tokio::{fs::File, io::AsyncWriteExt};
use std::cell::RefCell;
use std::io;
use std::rc::Rc;

pub trait FileStorer {
    fn write(&mut self, stream: Box<dyn Stream<Item = Result<Bytes, Error>>>) -> Result<String, Error>;
    fn read(&self, id: &str) -> Result<Box<dyn Stream<Item = Result<Bytes, Error>>>, Error>;
}

pub async fn upload(mut payload: Multipart, storer: Data<Rc<RefCell<dyn FileStorer>>>) -> Result<Json<()>, Error> {
    while let Some(field) = payload.try_next().await? {
        println!("{}", field.name());
        let stream = field.map_err(|e| Error::from(e));
        let id = storer.borrow_mut().write(Box::new(stream))?;
    }
    Ok(Json(()))
}
