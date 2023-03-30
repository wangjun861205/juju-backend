use crate::{context::UserInfo, error::Error};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    HttpMessage,
};
use sqlx::{query::QueryScalar, query_scalar, PgPool};
use std::future::Future;
use std::future::{ready, Ready};
use std::pin::Pin;
use std::task::Poll;

pub struct Author {
    db: PgPool,
    sql_stmt: String,
    path_arg_name: String,
}

impl Author {
    pub fn new(db: PgPool, sql_stmt: &str, path_arg_name: &str) -> Self {
        Self {
            db,
            sql_stmt: sql_stmt.into(),
            path_arg_name: path_arg_name.into(),
        }
    }
}

impl<S> Transform<S, ServiceRequest> for Author
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error>,
    S::Future: 'static,
{
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    type Response = S::Response;
    type Error = S::Error;
    type InitError = ();
    type Transform = AuthorMiddleware<S>;
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthorMiddleware {
            db: self.db.clone(),
            sql_stmt: self.sql_stmt.clone(),
            path_arg_name: self.path_arg_name.clone(),
            service,
        }))
    }
}

pub struct AuthorMiddleware<S> {
    db: PgPool,
    sql_stmt: String,
    path_arg_name: String,
    service: S,
}

impl<S> Service<ServiceRequest> for AuthorMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error>,
    S::Future: 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<ServiceResponse, Self::Error>>>>;
    fn poll_ready(&self, _: &mut core::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let user_info = req.extensions_mut().remove::<UserInfo>();
        let path = req.match_info().clone();
        if let Some(user_info) = user_info {
            req.extensions_mut().insert(user_info.clone());
            let uid = user_info.id;
            let next = self.service.call(req);
            if let Some(oid) = path.get(&self.path_arg_name) {
                if let Ok(oid) = oid.parse::<i32>() {
                    let stmt = self.sql_stmt.clone();
                    let db = self.db.clone();
                    return Box::pin(async move {
                        let q: QueryScalar<_, bool, _> = query_scalar(&stmt).bind(uid).bind(oid);
                        match db.acquire().await {
                            Ok(mut conn) => match q.fetch_one(&mut conn).await {
                                Ok(is_valid) => {
                                    if !is_valid {
                                        return Err(actix_web::error::ErrorForbidden("forbidden"));
                                    }
                                    next.await
                                }
                                Err(err) => Err(actix_web::error::ErrorInternalServerError(err)),
                            },
                            Err(err) => Err(actix_web::error::ErrorInternalServerError(err)),
                        }
                    });
                }
                return Box::pin(async move { Err(actix_web::error::ErrorBadRequest("invalid argument")) });
            }
            return Box::pin(async move {
                Err(actix_web::error::ErrorInternalServerError(Error::ServerError(
                    "authorizer middleware may mounted at a unsuitable position".into(),
                )))
            });
        }
        Box::pin(async move { Err(actix_web::error::ErrorUnauthorized("unauthorized")) })
    }
}
