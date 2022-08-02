#[macro_use]
extern crate diesel;
extern crate dotenvy;

use actix_session::{storage::RedisActorSessionStore, SessionMiddleware};
use actix_web::cookie::Key;
use actix_web::web::resource;

use diesel::{r2d2, r2d2::ConnectionManager, PgConnection};

use dotenvy::dotenv;
use std::env::set_var;

mod configuration;
mod db;
mod handlers;
mod models;
mod schema;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    set_var("RUST_LOG", "debug");
    env_logger::builder().format_timestamp(None).init();

    use configuration::Application;
    let settings = Application {
        redis_uri: "127.0.0.1:6379".to_string(),
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
    use handlers::pages;
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/", web::get().to(pages::index))
            .service(resource("/login_form").route(web::post().to(pages::login_form)))
            .service(resource("/login").route(web::get().to(pages::login)))
            .service(resource("/login_custom").route(web::get().to(pages::login_custom)))
            .service(resource("/show_login").route(web::get().to(pages::show_login)))
            .route("/process", web::post().to(pages::add_user))
            .route("/show_users", web::get().to(pages::show_users))
            .wrap(
                SessionMiddleware::builder(
                    RedisActorSessionStore::new(settings.redis_uri.clone()),
                    private_key.clone(),
                )
                .build(),
            )
    })
    .bind("127.0.0.1:8080")
    .expect("Can not bind to 127.0.0.1:8080")
    .run()
    .await
}
