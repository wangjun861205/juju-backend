use crate::actix_multipart::Multipart;
use crate::actix_web::web::{Data, Json, Path};
use crate::bytes::{BufMut, Bytes, BytesMut};
use crate::context::UserInfo;
use crate::error::Error;
use crate::futures_util::TryStreamExt;
use crate::UploadPath;
use actix_files as fs;
use sqlx::{query, query_as, PgPool};
use std::path::Path as FilePath;
pub trait FileStorer {
    fn write(&self, bytes: Bytes) -> Result<String, Error>;
    fn read(&self, fetch_code: &str) -> Result<Bytes, Error>;
}

pub async fn create<S: FileStorer>(me: UserInfo, mut payload: Multipart, storer: Data<S>, db: Data<PgPool>) -> Result<Json<Vec<String>>, Error> {
    let mut fetch_codes = Vec::new();
    while let Some(mut field) = payload.try_next().await? {
        let name = field.name().to_owned();
        let mut content = BytesMut::new();
        while let Some(b) = field.try_next().await? {
            content.put(b);
        }
        let fetch_code = storer.write(content.freeze())?;
        fetch_codes.push(fetch_code.clone());
        let mut conn = db.clone().acquire().await?;
        query("INSERT INTO uploaded_files (name, fetch_code, owner) VALUES ($1, $2, $3)")
            .bind(name)
            .bind(fetch_code)
            .bind(me.id)
            .execute(&mut conn)
            .await?;
    }
    Ok(Json(fetch_codes))
}

pub async fn fetch(me: UserInfo, fetch_code: Path<String>, db: Data<PgPool>, path: Data<UploadPath>) -> Result<fs::NamedFile, Error> {
    let mut conn = db.acquire().await?;
    let (ok,): (bool,) = query_as(
        "
    SELECT EXISTS(
        SELECT *
        FROM uploaded_files
        WHERE fetch_code = $1
        AND owner = $2)",
    )
    .bind(fetch_code.as_str())
    .bind(me.id)
    .fetch_one(&mut conn)
    .await?;
    if !ok {
        return Err(Error::BusinessError(format!("file not exists({})", fetch_code.into_inner())));
    }
    let f = fs::NamedFile::open(FilePath::new(&path.into_inner().0).join(fetch_code.into_inner()))?;
    Ok(f)
}
