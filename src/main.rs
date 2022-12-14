#[macro_use]
extern crate diesel;
extern crate dotenvy;
extern crate serde;

use actix_cors::Cors;
use actix_identity::IdentityMiddleware;
use actix_session::{storage::RedisActorSessionStore, SessionMiddleware};
use actix_web::{cookie::Key, http::header, web, web::resource, App, HttpServer};
use actix_web_httpauth::middleware::HttpAuthentication;

use diesel::{r2d2, r2d2::ConnectionManager, PgConnection};

use dotenvy::dotenv;

use configuration::Application;
use handlers::{pages, tests};

mod auth;
mod configuration;
mod db;
mod errors;
mod handlers;
mod models;
mod schema;
mod utils;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::builder().format_timestamp(None).init();

    let settings = Application::default();
    let private_key = Key::generate();

    let connection_manager = ConnectionManager::<PgConnection>::new(settings.database_url);
    let pool: models::Pool = r2d2::Pool::builder()
        .build(connection_manager)
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        // TODO change to better custom target
        //let cors = Cors::permissive();
        let cors = Cors::default()
            //.allow_any_origin()
            .allowed_origin("http://127.0.0.1:8080")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
            .allowed_header(header::CONTENT_TYPE)
            .supports_credentials()
            .disable_preflight()
            .max_age(3600);
        let auth = HttpAuthentication::bearer(auth::validator);
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
            .service(resource("/show_users.json").route(web::get().to(pages::show_users)))
            .service(
                web::scope("/test")
                    .route("html.html", web::get().to(tests::test_html))
                    .route("post.json", web::post().to(tests::test_post))
                    .route("get.json", web::get().to(tests::test_get))
                    .route("get_vec.json", web::get().to(tests::test_get_vec))
                    .route("login.json", web::post().to(tests::test_login))
                    .service(
                        web::scope("/auth")
                            .wrap(auth)
                            .route("admin.json", web::get().to(tests::test_admin_page)),
                    ),
            )
    })
    .bind("127.0.0.1:8084")
    .expect("Can not bind to 127.0.0.1:8084")
    .run()
    .await
}

// ------ Unit Tests ------
#[cfg(test)]
mod unit_tests {
    use std::convert::TryInto;

    use actix_web::{body::to_bytes, dev::Service, http, test, web, App};

    use crate::auth::UserPermissions;

    use super::*;
    use dotenvy::dotenv;
    use handlers::tests::SendMessageRequestBody;
    use log::debug;

    #[actix_web::test]
    async fn test_post() {
        let app = test::init_service(App::new().route("/", web::post().to(tests::test_post))).await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&SendMessageRequestBody {
                text: "my-name".to_owned(),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
        assert!(resp.status().is_success());

        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(
            body_bytes,
            web::Bytes::from(r##"{"ordinal_number":42,"text":"my-name"}"##)
        );
    }

    #[actix_web::test]
    async fn test_get() {
        // DEMO for POOL Env
        dotenv().ok();
        let settings = Application::default();
        let connection_manager = ConnectionManager::<PgConnection>::new(settings.database_url);
        let pool: models::Pool = r2d2::Pool::builder()
            .build(connection_manager)
            .expect("Failed to create pool.");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/", web::get().to(tests::test_get)),
        )
        .await;
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_get_vec() {
        let app =
            test::init_service(App::new().route("/", web::get().to(tests::test_get_vec))).await;
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
        assert!(resp.status().is_success());

        let body_bytes = to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(
            body_bytes,
            web::Bytes::from(
                r##"{"response":[{"ordinal_number":42,"text":"response"},{"ordinal_number":22,"text":"response2"}]}"##
            )
        );
    }

    #[actix_web::test]
    async fn test_login() {
        let app = test::init_service(
            App::new()
                .route("/", web::post().to(tests::test_login))
                .route("/login", web::post().to(tests::test_admin_page)),
        )
        .await;
        let req = test::TestRequest::post()
            .uri("/")
            .set_json(&UserPermissions {
                username: "my-name".to_owned(),
                permissions: Vec::from(["ADMIN_ROLE".to_string()]),
            })
            .to_request();
        let resp = test::call_service(&app, req).await;
        println!("{:?}", resp);
        assert!(resp.status().is_success());
    }
}
