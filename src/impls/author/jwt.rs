use crate::context::UserInfo;
use crate::core::tokener::{Payload, Tokener};
use crate::error::Error;
use actix_web::dev::{Service, ServiceRequest, Transform};
use actix_web::HttpMessage;
use futures::Future;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::pin::Pin;

pub struct JWT {
    secret: Vec<u8>,
}

impl JWT {
    pub fn new(secret: Vec<u8>) -> Self {
        Self { secret }
    }
}

impl<P> Tokener<P> for JWT
where
    P: Payload,
{
    fn gen_token(&self, payload: &P) -> Result<String, Error> {
        let header = Header::new(Algorithm::HS256);
        let key = EncodingKey::from_secret(&self.secret);
        let token = encode(&header, payload, &key)?;
        Ok(token)
    }
    fn verify_token(&self, token: &str) -> Result<P, Error> {
        let key = DecodingKey::from_secret(&self.secret);
        let validation = Validation::new(Algorithm::HS256);
        let payload = decode(token, &key, &validation)?;
        Ok(payload.claims)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claim {
    user: String,
    exp: i64,
}

impl Payload for Claim {
    fn user(&self) -> &str {
        &self.user
    }
}

pub struct JWTMiddleware {
    secrte: Vec<u8>,
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
    type InitError = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Transform, Self::InitError>>>>;
    fn new_transform(&self, service: S) -> Self::Future {
        let secret = self.secrte.clone();
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
            return Box::pin(async move { Err(Error::Unauthorized) });
        }
        let header = header.unwrap().to_owned();
        match header.to_str() {
            Err(e) => return Box::pin(async move { Err(anyhow::Error::new(e).into()) }),
            Ok(token) => match <JWT as Tokener<Claim>>::verify_token(&self.tokener, token) {
                Err(e) => return Box::pin(async move { Err(e) }),
                Ok(claim) => match claim.user.parse::<i32>() {
                    Err(e) => return Box::pin(async move { Err(anyhow::Error::new(e).into()) }),
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

#[cfg(test)]
mod test {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize)]
    struct Claim {
        user: String,
        exp: i64,
    }

    impl Payload for Claim {
        fn user(&self) -> &str {
            &self.user
        }
    }

    #[test]
    fn test_gen_and_verify_token() {
        let jwt = JWT::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]);
        let claim = Claim {
            user: "bear dad".into(),
            exp: chrono::offset::Utc::now().timestamp(),
        };
        let token = jwt.gen_token(&claim).unwrap();
        let c: Claim = jwt.verify_token(&token).unwrap();
        assert_eq!(claim.user, c.user);
    }

    #[test]
    fn test_different_tokens() {
        let jwt = JWT::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 0]);
        let claim_a = Claim {
            user: "a".into(),
            exp: chrono::offset::Utc::now().timestamp(),
        };
        let token_a = jwt.gen_token(&claim_a).unwrap();
        let claim_b = Claim {
            user: "b".into(),
            exp: chrono::offset::Utc::now().timestamp(),
        };
        let token_b = jwt.gen_token(&claim_b).unwrap();
        let c_a: Claim = jwt.verify_token(&token_a).unwrap();
        let c_b: Claim = jwt.verify_token(&token_b).unwrap();
        assert_eq!(c_a.user, claim_a.user);
        assert_eq!(c_b.user, claim_b.user);
    }
}
