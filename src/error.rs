use actix_web::ResponseError;

use crate::actix_web;
use crate::actix_web::http::{self, header};
use crate::chrono;
use crate::diesel::result::Error as DieselError;
use crate::dotenv::Error as DotError;
use crate::jsonwebtoken::errors::Error as JsonWebTokenError;
use crate::r2d2::Error as R2D2Error;
use crate::thiserror::Error as ThisError;
use std::num;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("database error: {0}")]
    DatabaseError(#[from] DieselError),

    #[error("http error")]
    ActixError(#[from] actix_web::error::Error),

    #[error("dotenv error")]
    DotEnvError(#[from] DotError),

    #[error("jwt error")]
    JWTError(#[from] JsonWebTokenError),

    #[error("r2d2 error")]
    PoolError(#[from] R2D2Error),

    #[error("bussiness error: {0}")]
    BusinessError(String),

    #[error("parse int error: {0}")]
    ParseIntError(#[from] num::ParseIntError),

    #[error("http error")]
    HttpError(#[from] http::Error),

    #[error("header error")]
    HeaderError(#[from] header::ToStrError),

    #[error("failed to parse date")]
    ParseDate(#[from] chrono::ParseError),

    #[error("server error: {0}")]
    ServerError(String),
}

impl ResponseError for Error {}
