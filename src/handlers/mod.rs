pub mod answer;
pub mod authorizer;
pub mod date;
pub mod option;
pub mod organization;
pub mod question;
pub mod upload;
pub mod user;
pub mod vote;

use actix_web::http::StatusCode;
use rand::Rng;
use sqlx::{query, query_as, PgPool};
use std::ops::Add;

use crate::actix_web::{
    cookie::Cookie,
    web::{Data, Json},
    HttpResponse,
};

use crate::dotenv;
use crate::error::Error;
use crate::hex::ToHex;
use crate::jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use crate::middleware::jwt::{Claim, JWT_SECRET, JWT_TOKEN};
use crate::models::*;
use crate::rand::thread_rng;
use crate::serde::Deserialize;
use crate::sha2::{Digest, Sha256};

#[derive(Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}

fn hash_password(pass: &str, slt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(pass);
    hasher.update(slt);
    hasher.finalize().encode_hex()
}

pub async fn login(Json(Login { username, password }): Json<Login>, db: Data<PgPool>) -> Result<HttpResponse, Error> {
    let mut conn = db.acquire().await?;
    let user: User = query_as(r#"SELECT * FROM users WHERE phone = $1 OR email = $1"#).bind(&username).fetch_one(&mut conn).await?;
    if hash_password(&password, &user.salt) != user.password {
        return Ok(HttpResponse::build(StatusCode::FORBIDDEN).finish());
    }
    let claim = Claim {
        uid: user.id,
        exp: chrono::Utc::now().add(chrono::Duration::days(30)).timestamp() as usize,
    };
    let secret = dotenv::var(JWT_SECRET)?;
    let token = encode(&Header::new(Algorithm::HS256), &claim, &EncodingKey::from_secret(secret.as_bytes()))?;

    Ok(HttpResponse::build(StatusCode::OK).cookie(Cookie::new(JWT_TOKEN, token)).finish())
}

fn random_salt() -> String {
    let chars = vec![
        '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B',
        'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    ];
    let mut slt = String::new();
    let mut rng = thread_rng();
    for _ in 0..32 {
        let i = rng.gen_range(0..61_usize);
        slt.push(chars[i]);
    }
    slt
}

#[derive(Debug, Clone, Deserialize)]
pub struct Signup {
    nickname: String,
    phone: String,
    email: String,
    password: String,
    invite_code: String,
}

pub async fn signup(
    Json(Signup {
        nickname,
        phone,
        email,
        password,
        invite_code,
    }): Json<Signup>,
    db: Data<PgPool>,
) -> Result<HttpResponse, Error> {
    let mut tx = db.begin().await?;
    let (deleted,): (i64,) = query_as("DELETE FROM invite_codes WHERE code = $1").bind(invite_code).fetch_one(&mut tx).await?;
    if deleted == 0 {
        return Err(Error::BusinessError("invalid invite code".into()));
    }
    let slt = random_salt();
    query("INSERT INTO users (nickname, phone, email, password, salt) VALUES ($1, $2, $3, $4, $5)")
        .bind(nickname)
        .bind(phone)
        .bind(email)
        .bind(hash_password(&password, &slt))
        .bind(slt)
        .execute(&mut tx)
        .await?;
    tx.commit().await?;
    Ok(HttpResponse::build(StatusCode::OK).finish())
}
