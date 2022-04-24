use super::DB;
use crate::actix_multipart::Multipart;
use crate::actix_web::web::{block, Data, Json};
use crate::bytes::Bytes;
use crate::diesel::{insert_into, ExpressionMethods, RunQueryDsl};
use crate::error::Error;
use crate::futures_util::{Stream, StreamExt, TryStreamExt};
use crate::schema::*;
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

pub trait FileStorer {
    fn write(&mut self, bytes: Vec<u8>) -> Result<String, Error>;
    fn read(&self, id: &str) -> Result<Vec<u8>, Error>;
}

pub async fn create(mut payload: Multipart, storer: Data<Rc<RefCell<dyn FileStorer>>>, db: DB) -> Result<Json<Vec<i32>>, Error> {
    let mut ids = Vec::new();
    while let Some(mut field) = payload.try_next().await? {
        let name = field.name().to_owned();
        let mut content = Vec::new();
        while let Some(b) = field.try_next().await? {
            content.append(&mut b.to_vec());
        }
        let fetch_code = storer.borrow_mut().write(content)?;
        let db = db.clone();
        let id = block(move || {
            let conn = db.get().unwrap();
            insert_into(uploaded_files::table)
                .values((uploaded_files::name.eq(name), uploaded_files::fetch_code.eq(fetch_code)))
                .returning(uploaded_files::id)
                .get_result::<i32>(&conn)
        })
        .await??;
        ids.push(id);
    }
    Ok(Json(ids))
}
