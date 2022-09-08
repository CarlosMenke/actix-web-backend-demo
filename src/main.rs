#[macro_use]
extern crate diesel;
extern crate dotenvy;
extern crate serde;

use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::{storage::RedisActorSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::web::resource;

use diesel::{r2d2, r2d2::ConnectionManager, PgConnection};

use dotenvy::dotenv;
use std::env::set_var;

mod configuration;
mod db;
mod errors;
mod handlers;
mod models;
mod schema;
mod utils;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    set_var("RUST_LOG", "debug");
    env_logger::builder().format_timestamp(None).init();

    use configuration::Application;
    let settings = Application {
        redis_uri: "127.0.0.1:6379".to_string(),
        domain: env::var("DOMAIN").unwrap_or_else(|_| "localhost".to_string()),
    };
    let private_key = Key::generate();

    use std::env;
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    let pool: models::Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    // TODO move config data to config struct
    use actix_web::{web, App, HttpServer};
    use handlers::{pages, tests};
    HttpServer::new(move || {
        // TODO change to better custom target
        let cors = Cors::permissive();
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors)
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(
                    RedisActorSessionStore::new(settings.redis_uri.clone()),
                    private_key.clone(),
                )
                .build(),
            )
            .service(
                resource("/")
                    .route(web::get().to(pages::index))
                    .route(web::post().to(pages::add_user)),
            )
            .service(
                resource("/login")
                    .route(web::get().to(pages::login))
                    .route(web::post().to(pages::login_form)),
            )
            .service(resource("/show_login").route(web::get().to(pages::show_login)))
            .service(resource("/logout").route(web::get().to(pages::logout)))
            .route("/show_users.json", web::get().to(pages::show_users))
            .route("/test_html", web::get().to(tests::test_html))
            .route("/test_post.json", web::post().to(tests::test_post))
            .route("/test_get.json", web::get().to(tests::test_get))
            .route("/test_get_vec.json", web::get().to(tests::test_get_vec))
    })
    .bind("127.0.0.1:8084")
    .expect("Can not bind to 127.0.0.1:8084")
    .run()
    .await
}
