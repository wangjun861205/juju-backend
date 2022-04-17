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

mod authorizer;
mod context;
mod error;
mod handlers;
mod middleware;
pub mod models;
pub mod privilege;
pub mod request;
pub mod response;
mod schema;

use actix_web::web::{delete, get, post, put, resource, scope, Data};
use actix_web::HttpServer;
use authorizer::PgAuthorizer;
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
            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(PgAuthorizer::new(pool.clone())))
            .service(
                scope("")
                    .service(resource("login").route(post().to(handlers::login)))
                    .service(resource("signup").route(post().to(handlers::signup)))
                    .service(
                        scope("")
                            .wrap(JWT {})
                            .service(
                                scope("organizations")
                                    .route("", get().to(handlers::organization::list))
                                    .route("", post().to(handlers::organization::create))
                                    .service(
                                        scope("{organization_id}")
                                            .route("", get().to(handlers::organization::detail))
                                            .route("", put().to(handlers::organization::update))
                                            .route("", delete().to(handlers::organization::delete_organization))
                                            .service(scope("votes").route("", post().to(handlers::vote::create)).route("", get().to(handlers::vote::list)))
                                            .service(
                                                scope("users")
                                                    .route("", post().to(handlers::organization::add_users))
                                                    .route("", get().to(handlers::organization::users::<PgAuthorizer>)),
                                            ),
                                    ),
                            )
                            .service(
                                scope("votes").route("", post().to(handlers::vote::create)).service(
                                    scope("{vote_id}")
                                        .route("", get().to(handlers::vote::detail))
                                        .route("", put().to(handlers::vote::update))
                                        .route("", delete().to(handlers::vote::delete_vote))
                                        .service(
                                            scope("date_ranges")
                                                .route("", get().to(handlers::date::date_range_list))
                                                .route("", put().to(handlers::date::submit_date_ranges))
                                                .service(
                                                    scope("report")
                                                        .route("year", get().to(handlers::date::year_report))
                                                        .route("month", get().to(handlers::date::month_report)),
                                                ),
                                        )
                                        .service(
                                            scope("questions")
                                                .route("", post().to(handlers::question::create))
                                                .route("", get().to(handlers::question::list))
                                                .route("report", get().to(handlers::vote::question_reports)),
                                        ),
                                ),
                            )
                            .service(
                                scope("questions").service(
                                    scope("{question_id}")
                                        .route("", get().to(handlers::question::detail))
                                        .route("", delete().to(handlers::question::delete))
                                        .service(scope("options").route("", post().to(handlers::option::add_opts)).route("", get().to(handlers::option::list)))
                                        .service(scope("answers").route("", get().to(handlers::answer::answer_list)).route("", put().to(handlers::answer::submit_answer))), // .service(scope("report").route("", get().to(handlers::question::gen_question_report))),
                                ),
                            )
                            .service(scope("users").route("", get().to(handlers::user::list))),
                    ),
            )
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
