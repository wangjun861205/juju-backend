use actix_web::error::ErrorBadRequest;
use serde::{Deserialize, Serialize};

use crate::actix_web::{
    dev::{Service, ServiceRequest, Transform},
    error::ErrorUnauthorized,
    Error, HttpMessage,
};
use crate::context::UserInfo;
use crate::core::tokener::{Payload, Tokener};
use crate::impls::tokener::jwt::JWT;
use std::future::Future;
use std::pin::Pin;

pub static JWT_TOKEN: &str = "JWT_TOKEN";
pub static JWT_SECRET: &str = "JWT_SECRET";

#[derive(Debug, Deserialize, Serialize)]
pub struct Claim {
    pub user: String,
    pub exp: i64,
}

impl Payload for Claim {
    fn user(&self) -> &str {
        &self.user
    }
}

pub(crate) struct JWTMiddleware {
    secret: Vec<u8>,
}

impl JWTMiddleware {
    pub fn new(secret: Vec<u8>) -> Self {
        Self { secret }
    }
}

impl<S> Transform<S, ServiceRequest> for JWTMiddleware
where
    S: Service<ServiceRequest> + 'static,
    S::Future: 'static,
    S::Error: Into<Error>,
{
    type Error = Error;
    type Response = S::Response;
    type Transform = JWTService<S>;
    type InitError = ();
    type Future = Pin<Box<dyn Future<Output = Result<Self::Transform, Self::InitError>>>>;
    fn new_transform(&self, service: S) -> Self::Future {
        let secret = self.secret.clone();
        Box::pin(async move {
            Ok(JWTService {
                tokener: JWT::new(secret),
                next_service: service,
            })
        })
    }
}

pub struct JWTService<S> {
    tokener: JWT,
    next_service: S,
}

impl<S> Service<ServiceRequest> for JWTService<S>
where
    S: Service<ServiceRequest>,
    S::Future: 'static,
    S::Error: Into<Error>,
{
    type Response = S::Response;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
    fn poll_ready(&self, ctx: &mut core::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.next_service.poll_ready(ctx).map_err(|e| e.into())
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let header = req.headers().get("Authorization");
        if header.is_none() {
            return Box::pin(async move { Err(ErrorUnauthorized("no token in header")) });
        }
        let header = header.unwrap().to_owned();
        match header.to_str() {
            Err(e) => return Box::pin(async move { Err(ErrorUnauthorized(e)) }),
            Ok(token) => match <JWT as Tokener<Claim>>::verify_token(&self.tokener, token) {
                Err(e) => return Box::pin(async move { Err(ErrorUnauthorized(e)) }),
                Ok(claim) => match claim.user.parse::<i32>() {
                    Err(e) => return Box::pin(async move { Err(ErrorUnauthorized(e)) }),
                    Ok(id) => {
                        req.extensions_mut().insert(UserInfo { id });
                    }
                },
            },
        }

        let res_fut = self.next_service.call(req);
        Box::pin(async move {
            let resp = res_fut.await.map_err(|e| e.into())?;
            Ok(resp)
        })
    }
}
