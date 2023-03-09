use super::DB;
use crate::actix_multipart::Multipart;
use crate::actix_web::web::{block, Data, Json, Path, Query};
use crate::bytes::{BufMut, Bytes, BytesMut};
use crate::context::UserInfo;
use crate::diesel::{
    dsl::{exists, select},
    insert_into, BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl,
};
use crate::error::Error;
use crate::futures_util::TryStreamExt;
use crate::schema::*;
use crate::UploadPath;
use actix_files as fs;
use actix_web::get;
use std::path::Path as FilePath;
pub trait FileStorer {
    fn write(&self, bytes: Bytes) -> Result<String, Error>;
    fn read(&self, fetch_code: &str) -> Result<Bytes, Error>;
}

pub async fn create<S: FileStorer>(me: UserInfo, mut payload: Multipart, storer: Data<S>, db: DB) -> Result<Json<Vec<String>>, Error> {
    let mut fetch_codes = Vec::new();
    while let Some(mut field) = payload.try_next().await? {
        let name = field.name().to_owned();
        let mut content = BytesMut::new();
        while let Some(b) = field.try_next().await? {
            content.put(b);
        }
        let fetch_code = storer.write(content.freeze())?;
        fetch_codes.push(fetch_code.clone());
        let conn = db.clone().get()?;
        block(move || {
            insert_into(uploaded_files::table)
                .values((uploaded_files::name.eq(name), uploaded_files::fetch_code.eq(fetch_code), uploaded_files::ownner.eq(me.id)))
                .execute(&mut conn)
        })
        .await??;
    }
    Ok(Json(fetch_codes))
}

pub async fn fetch(me: UserInfo, fetch_code: Path<String>, db: DB, path: Data<UploadPath>) -> Result<fs::NamedFile, Error> {
    let conn = db.get()?;
    let ok: bool = select(exists(
        uploaded_files::table.filter(uploaded_files::fetch_code.eq(fetch_code.clone()).and(uploaded_files::ownner.eq(me.id))),
    ))
    .get_result(&mut conn)?;
    if !ok {
        return Err(Error::BusinessError(format!("file not exists({})", fetch_code.into_inner())));
    }
    let f = fs::NamedFile::open(FilePath::new(&path.into_inner().0).join(fetch_code.into_inner()))?;
    Ok(f)
}
