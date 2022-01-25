#[macro_use]
extern crate diesel;
extern crate actix_web;
extern crate casbin;
extern crate chrono;
extern crate diesel_derive_enum;
extern crate dotenv;
extern crate env_logger;
extern crate hex;
extern crate jsonwebtoken;
extern crate r2d2;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate sha2;
extern crate thiserror;
extern crate tokio;

mod context;
mod error;
mod handlers;
mod middleware;
pub mod models;
pub mod privilege;
pub mod response;
mod schema;

use actix_web::web::{put, resource};
use actix_web::HttpServer;
use diesel::pg::PgConnection;
use middleware::jwt::JWT;
use r2d2::Pool;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let manager = diesel::r2d2::ConnectionManager::<PgConnection>::new(dotenv::var("DATABASE_URL").unwrap());
    let pool = Pool::new(manager).unwrap();
    HttpServer::new(move || {
        actix_web::App::new()
            .wrap(actix_web::middleware::Logger::default())
            .data(pool.clone())
            .route("/login", actix_web::web::post().to(handlers::login))
            .route("/signup", actix_web::web::post().to(handlers::signup))
            .service(
                actix_web::web::scope("/organizations")
                    .wrap(JWT {})
                    .route("", actix_web::web::get().to(handlers::organization_list))
                    .route("", actix_web::web::post().to(handlers::create_organization))
                    .route("/{organization_id}", actix_web::web::get().to(handlers::organization_detail))
                    .route("/{organization_id}", actix_web::web::delete().to(handlers::delete_organization))
                    .service(
                        actix_web::web::scope("/{organizations_id}/votes")
                            .route("", actix_web::web::get().to(handlers::vote_list))
                            .route("/{vote_id}", actix_web::web::get().to(handlers::vote_detail))
                            .route("/{vote_id}", actix_web::web::put().to(handlers::update_vote))
                            .service(
                                actix_web::web::scope("/{vote_id}/questions")
                                    .route("", actix_web::web::post().to(handlers::add_question))
                                    .route("/{question_id}/opts", actix_web::web::post().to(handlers::add_opts)),
                            ),
                    ),
            )
            .service(actix_web::web::resource("/votes").wrap(JWT {}).route(actix_web::web::post().to(handlers::create_vote)))
            .service(
                actix_web::web::resource("/questions/{qid}/options/{oid}/answers")
                    .wrap(JWT {})
                    .route(actix_web::web::post().to(handlers::submit_answer)),
            )
            .service(resource("/votes/{vid}/dates").wrap(JWT {}).route(put().to(handlers::submit_dates)))
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
