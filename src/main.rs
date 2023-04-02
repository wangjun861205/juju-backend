#![feature(async_fn_in_trait)]

extern crate actix_multipart;
extern crate actix_web;
extern crate bytes;
extern crate casbin;
extern crate chrono;
extern crate default;
extern crate dotenv;
extern crate env_logger;
extern crate futures;
extern crate futures_util;
extern crate hex;
extern crate hex_literal;
extern crate itertools;
extern crate jsonwebtoken;
extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate sha2;
extern crate sqlx;
extern crate sqlx_insert;
extern crate thiserror;
extern crate tokio;

mod authorizer;
mod context;
mod error;
mod handlers;
mod middlewares;
pub mod models;
pub mod privilege;
pub mod request;
pub mod response;
mod storer;

use actix_web::web::{delete, get, post, put, resource, scope, Data};
use actix_web::HttpServer;
use authorizer::PgAuthorizer;
use middlewares::authorizer::Author;
use middlewares::jwt::Jwt;
use sqlx::postgres::PgPoolOptions;

#[derive(Debug, Clone)]
pub struct UploadPath(pub String);

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=info");
    env_logger::init();
    let upload_path = dotenv::var("UPLOAD_PATH").expect("environment variable UPLOAD_PATH not been set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost/juju")
        .await
        .expect("failed to connect to database");
    HttpServer::new(move || {
        actix_web::App::new()
            .wrap(actix_web::middleware::Logger::default())
            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(PgAuthorizer::new(pool.clone())))
            .app_data(Data::new(storer::LocalStorer::new(&upload_path)))
            .app_data(Data::new(UploadPath(upload_path.clone())))
            .service(
                scope("")
                    .service(resource("login").route(post().to(handlers::login)))
                    .service(resource("signup").route(post().to(handlers::signup)))
                    .service(
                        scope("")
                        .wrap(Jwt {})
                            .service(
                                scope("upload")
                                    .route("", post().to(handlers::upload::create::<storer::LocalStorer>))
                                    .route("", get().to(handlers::upload::fetch)),
                            )
                            .service(
                                scope("organizations")
                                    .route("", get().to(handlers::organization::list))
                                    .route("", post().to(handlers::organization::create))
                                    .service(
                                        scope("{organization_id}")
                                            .wrap(Author::new(
                                                pool.clone(),
                                                "SELECT EXISTS(SELECT id FROM users_organizations WHERE user_id = $1 AND organization_id = $2)",
                                                "organization_id",
                                            ))
                                            .route("", get().to(handlers::organization::detail))
                                            .route("", put().to(handlers::organization::update))
                                            .route("", delete().to(handlers::organization::delete_organization))
                                            .service(
                                                scope("votes")
                                                .route("", post().to(handlers::vote::create))
                                                .route("", get().to(handlers::organization::votes)))
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
                                        .wrap(Author::new(
                                            pool.clone(),
                                            "SELECT EXISTS(SELECT uo.id FROM users_organizations AS uo JOIN votes AS v ON uo.organization_id = v.organization_id WHERE uo.user_id = $1 AND v.id = $2)",
                                            "vote_id",
                                        ))
                                        .route("", get().to(handlers::vote::detail))
                                        .route("", put().to(handlers::vote::update))
                                        .route("", delete().to(handlers::vote::delete_vote))
                                        .route("questions_with_options", get().to(handlers::question::questions_with_options_by_vote_id))
                                        .route("question_ids", get().to(handlers::vote::question_ids))
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
                                                .route("", get().to(handlers::vote::questions))
                                                .route("report", get().to(handlers::vote::question_reports)),
                                        )
                                ),
                            )
                            .service(
                                scope("questions").service(
                                    scope("{question_id}")
                                        .wrap(Author::new(
                                            pool.clone(),
                                            "SELECT EXISTS(SELECT uo.id FROM users_organizations AS uo JOIN votes AS v ON uo.organization_id = v.organization_id JOIN questions AS q ON v.id = q.vote_id WHERE uo.user_id = $1 AND q.id = $2)",
                                            "question_id"
                                        ))
                                        .route("", get().to(handlers::question::detail))
                                        .route("", delete().to(handlers::question::delete))
                                        .service(
                                            scope("options")
                                            .route("", post().to(handlers::option::add_opts))
                                            .route("", get().to(handlers::question::options))
                                        )
                                        .service(
                                            scope("answers")
                                            .route("", get().to(handlers::question::answers))
                                            .route("", put().to(handlers::answer::submit_answer))
                                        ),
                                ),
                            )
                            .service(
                                scope("options")
                                .service(
                                    scope("{option_id}")
                                    .wrap(Author::new(
                                        pool.clone(),
                                        "SELECT EXISTS(
                                            SELECT uo.id
                                            FROM users_organizations AS uo
                                            JOIN votes AS v ON uo.organization_id = v.organization_id
                                            JOIN questions AS q ON v.id = q.vote_id
                                            JOIN options AS o ON q.id = o.question_id
                                            WHERE uo.user_id = $1 AND o.id = $2)",
                                        "option_id"
                                    ))
                                    .route("", delete().to(handlers::option::delete))
                                )
                            )
                            .service(scope("users").route("", get().to(handlers::user::list))),
                    ),
            )
    })
    .bind(("0.0.0.0", 8000))?
    .run()
    .await
}
