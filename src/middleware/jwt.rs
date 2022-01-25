use serde::{Deserialize, Serialize};

use crate::actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::{ErrorInternalServerError, ErrorUnauthorized},
    http::{HeaderName, HeaderValue},
    Error, HttpMessage,
};
use crate::context::UserInfo;
use crate::dotenv;
use crate::jsonwebtoken;
use std::future::{ready, Future, Ready};
use std::pin::Pin;
use std::str::FromStr;

pub static JWT_TOKEN: &str = "JWT_TOKEN";
pub static JWT_SECRET: &str = "JWT_SECRET";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    pub uid: i32,
}

pub struct JWT;

impl<S> Transform<S> for JWT
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = JWTService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JWTService { service: service }))
    }
}

pub struct JWTService<S> {
    service: S,
}

impl<S> Service for JWTService<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Error>>>>;

    fn poll_ready(&mut self, ctx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }
    fn call(&mut self, mut req: Self::Request) -> Self::Future {
        match req.cookie(JWT_TOKEN) {
            None => return Box::pin(async move { Err(ErrorUnauthorized("unauthorized")) }),
            Some(jwt) => match dotenv::var(JWT_SECRET) {
                Ok(sct) => {
                    match jsonwebtoken::decode::<Claim>(
                        jwt.value(),
                        &jsonwebtoken::DecodingKey::from_secret(sct.as_bytes()),
                        &jsonwebtoken::Validation {
                            algorithms: vec![jsonwebtoken::Algorithm::HS256],
                            validate_exp: false,
                            ..Default::default()
                        },
                    ) {
                        Ok(c) => {
                            req.extensions_mut().insert(UserInfo { id: c.claims.uid });
                        }
                        Err(e) => {
                            println!("{}", e);
                            return Box::pin(async move { Err(ErrorUnauthorized("unauthorized")) });
                        }
                    }
                }
                Err(_) => return Box::pin(async move { Err(ErrorInternalServerError("internal server error")) }),
            },
        }
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
