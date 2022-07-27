#[macro_use]
extern crate diesel;
extern crate dotenvy;

use crate::configuration::Settings;
use actix_session::storage::RedisSessionStore;
use actix_web::cookie::Key;
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, Secret};

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

    //let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());
    //let message_store = CookieMessageStore::builder(secret_key.clone()).build();
    //let secret_key = Key::from(hmac_secret.expose_secret().as_bytes());

    use actix_web::{web, App, HttpServer};
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(handlers::pages::index))
            .route("/login_form", web::post().to(handlers::pages::login_form))
            .route("/login", web::get().to(handlers::pages::login))
            .route("/process", web::post().to(handlers::pages::add_user))
            .route("/show_users", web::get().to(handlers::pages::show_users))
    })
    .bind("127.0.0.1:8080")
    .expect("Can not bind to 127.0.0.1:8080")
    .run()
    .await
}
