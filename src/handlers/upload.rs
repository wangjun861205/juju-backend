use super::DB;
use crate::actix_multipart::Multipart;
use crate::actix_web::web::{block, Data, Json};
use crate::bytes::{BufMut, Bytes, BytesMut};
use crate::diesel::{insert_into, ExpressionMethods, RunQueryDsl};
use crate::error::Error;
use crate::futures_util::TryStreamExt;
use crate::schema::*;
use std::cell::RefCell;
use std::rc::Rc;

pub trait FileStorer {
    fn write(&self, bytes: Bytes) -> Result<String, Error>;
    fn read(&self, id: &str) -> Result<Bytes, Error>;
}

pub async fn create<S: FileStorer>(mut payload: Multipart, storer: Data<S>, db: DB) -> Result<Json<Vec<i32>>, Error> {
    let mut ids = Vec::new();
    while let Some(mut field) = payload.try_next().await? {
        let name = field.name().to_owned();
        let mut content = BytesMut::new();
        while let Some(b) = field.try_next().await? {
            content.put(b);
        }
        let fetch_code = storer.write(content.freeze())?;
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
