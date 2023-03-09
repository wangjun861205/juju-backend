use actix_web::ResponseError;

use crate::actix_multipart::MultipartError;
use crate::actix_web;
use crate::actix_web::error::BlockingError;
use crate::actix_web::http::{header, Error as HTTPError};
use crate::chrono;
use crate::dotenv::Error as DotError;
use crate::jsonwebtoken::errors::Error as JsonWebTokenError;
use crate::thiserror::Error as ThisError;
use std::io::Error as IOError;
use std::num;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("sqlx error")]
    SqlxError(#[from] sqlx::error::Error),

    #[error("http error")]
    ActixError(#[from] actix_web::error::Error),

    #[error("dotenv error")]
    DotEnvError(#[from] DotError),

    #[error("jwt error")]
    JWTError(#[from] JsonWebTokenError),

    #[error("bussiness error: {0}")]
    BusinessError(String),

    #[error("parse int error: {0}")]
    ParseIntError(#[from] num::ParseIntError),

    #[error("http error")]
    HTTPError(String),

    #[error("header error")]
    HeaderError(#[from] header::ToStrError),

    #[error("failed to parse date")]
    ParseDate(#[from] chrono::ParseError),

    #[error("server error: {0}")]
    ServerError(String),

    #[error("io error")]
    IOError(#[from] IOError),
}

impl ResponseError for Error {}

impl From<MultipartError> for Error {
    fn from(err: MultipartError) -> Self {
        Self::HTTPError(format!("{err:?}"))
    }
}

impl From<HTTPError> for Error {
    fn from(err: HTTPError) -> Self {
        Self::HTTPError(format!("{err:?}"))
    }
}

impl From<BlockingError> for Error {
    fn from(e: BlockingError) -> Self {
        Self::ServerError(format!("{:?}", e))
    }
}
