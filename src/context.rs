use crate::actix_web::FromRequest;
use crate::actix_web::{self, Error};
use std::future::{ready, Ready};

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub id: i32,
}

impl FromRequest for UserInfo {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();
    fn from_request(req: &actix_web::HttpRequest, payload: &mut actix_web::dev::Payload) -> Self::Future {
        if let Some(user) = req.extensions().get::<Self>() {
            ready(Ok(user.clone()))
        } else {
            ready(Err(actix_web::error::ErrorUnauthorized("")))
        }
    }
}
