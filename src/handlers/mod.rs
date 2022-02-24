pub mod answer;
pub mod date;
pub mod option;
pub mod organization;
pub mod question;
pub mod vote;

use actix_web::http::StatusCode;
use diesel::{Connection, QueryDsl};
use rand::Rng;

use crate::actix_web::{
    cookie::Cookie,
    web::{Data, Json},
    HttpResponse,
};

use crate::diesel::{pg::PgConnection, r2d2::ConnectionManager, ExpressionMethods, RunQueryDsl};
use crate::dotenv;
use crate::error::Error;
use crate::hex::ToHex;
use crate::jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use crate::middleware::jwt::{Claim, JWT_SECRET, JWT_TOKEN};
use crate::models::*;
use crate::r2d2::Pool;
use crate::rand::thread_rng;
use crate::schema::users::dsl::*;
use crate::serde::Deserialize;
use crate::sha2::{Digest, Sha256};

type DB = Data<Pool<ConnectionManager<PgConnection>>>;

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

pub async fn login(body: Json<Login>, db: Data<Pool<ConnectionManager<PgConnection>>>) -> Result<HttpResponse, Error> {
    let conn = db.get()?;
    let l = users.filter(phone.eq(&body.0.username)).or_filter(email.eq(&body.0.username)).load::<User>(&conn)?;
    if l.is_empty() {
        return Ok(HttpResponse::build(StatusCode::FORBIDDEN).finish());
    }
    if hash_password(&body.0.password, &l[0].salt) != l[0].password {
        return Ok(HttpResponse::build(StatusCode::FORBIDDEN).finish());
    }
    let claim = Claim { uid: l[0].id };
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
        let i = rng.gen_range(0, 61_usize);
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

pub async fn signup(Json(req): Json<Signup>, db: Data<Pool<ConnectionManager<PgConnection>>>) -> Result<HttpResponse, Error> {
    use crate::schema::invite_codes::dsl::*;
    let conn = db.get()?;
    conn.transaction::<(), Error, _>(|| {
        let deleted = diesel::delete(invite_codes.filter(code.eq(&req.invite_code))).execute(&conn)?;
        if deleted == 0 {
            return Err(Error::BusinessError("invalid invite code".into()));
        }
        let slt = random_salt();
        let insertion = crate::models::UserInsertion {
            nickname: req.nickname,
            phone: req.phone,
            email: req.email,
            password: hash_password(&req.password, &slt),
            salt: slt,
        };
        diesel::insert_into(users).values(insertion).execute(&conn)?;
        Ok(())
    })?;
    Ok(HttpResponse::build(StatusCode::OK).finish())
}
