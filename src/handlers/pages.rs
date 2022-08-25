use crate::db::users::{
    check_login, get_user as DBget_user, insert_user as DBinsert_user, show_users as DBshow_users,
};
use crate::models::Pool;
use actix_files::NamedFile;
use actix_identity::Identity;
use actix_session::{Session, SessionGetError};
use actix_web::dev::Service;
use actix_web::http::header::LOCATION;
use actix_web::{web, Error, HttpMessage, HttpRequest, HttpResponse, Result};
use diesel::PgConnection;
use log::{debug, error, info};
use serde::Deserialize;
use std::path::PathBuf;

use crate::errors::ServiceError;

//use crate::schema::user::password;

#[derive(Deserialize)]
pub struct NewUser {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct Login {
    username: String,
    password: String,
}

pub async fn index(_req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = "./files/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

pub async fn login(_req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = "./files/login.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

pub async fn login_form(
    req: HttpRequest,
    form: web::Form<Login>,
    pool: web::Data<Pool>,
    //id: Option<Identity>,
    session: Session,
) -> HttpResponse {
    //let c: Option<String> = session.get::<String>("user_id").unwrap();

    let name = form.username.to_string();
    let pwd = form.password.to_string();

    info!(
        "Login Attempt: \t\t\tUsr: {:?}\t\tPassword: {:?}",
        name, pwd
    );

    let connection: &mut PgConnection = &mut pool.get().unwrap();

    let result = match check_login(connection, &name, &pwd) {
        Ok(r) => r,
        Err(_) => {
            return HttpResponse::Ok().body(format!("username: {} password {:?}", &name, pwd))
        }
    };
    if result == false {
        return HttpResponse::Ok().body(format!("username: {} password {:?}", &name, pwd));
        //HttpResponse::Ok().body(format!("username: {} password {:?}", &name, pwd))
        //HttpResponse::Ok()
        //.content_type(ContentType::html())
        //.body(include_str!("../../files/login.html"))
    } else {
        match DBget_user(connection, &name, &pwd) {
            Ok(user) => {
                info!("logged in");
                Identity::login(&req.extensions_mut(), user.id.to_string().into());
                session.insert("username", &user.username);
                session.renew();
                HttpResponse::SeeOther()
                    .insert_header((LOCATION, "/show_login"))
                    .finish()
            }
            Err(_) => {
                error!("User with name {:?} not found", &name);
                return HttpResponse::Ok().body(format!("Failed to find user."));
            }
        }
    }
}

pub async fn add_user(
    pool: web::Data<Pool>,
    info: web::Form<NewUser>,
) -> Result<HttpResponse, ServiceError> {
    let connection: &mut PgConnection = &mut pool.get().unwrap();

    let name = info.username.to_string();
    let pwd = info.password.to_string();

    DBinsert_user(connection, &name, &pwd)?;
    Ok(HttpResponse::Ok().body(format!("username: {}", name)))
}

pub async fn show_users(pool: web::Data<Pool>, _rew: HttpRequest) -> HttpResponse {
    let connection: &mut PgConnection = &mut pool.get().unwrap();

    let result = DBshow_users(connection);
    let mut response = String::new();

    for user in result {
        response.push_str(&format!(
            "User id '{}' with Name '{}' and Pwd '{}'",
            user.id, user.username, user.password,
        ));
    }
    HttpResponse::Ok().json(format!("{:?}", response))
}

pub async fn show_login(
    id_option: Option<Identity>,
    session: Session,
) -> Result<HttpResponse, ServiceError> {
    let id = if let Some(id_option) = id_option {
        format!("{:?}", id_option.id())
    } else {
        return Err(ServiceError::Unauthorized);
    };

    let username = match session.get::<String>("username") {
        Ok(name) => name,
        Err(SessionGetError) => {
            return Err(ServiceError::InternalServerError(
                "Session not found".to_string(),
            ))
        }
    };

    let name = if let Some(username) = username {
        format!("{:?}", username)
    } else {
        return Err(ServiceError::Unauthorized);
    };

    Ok(HttpResponse::Ok().body(format!("user_id {:?} username {:?}", id, name)))
}

pub async fn logout(id: Identity) -> HttpResponse {
    debug!("Logout");
    let user = id.id().unwrap();
    id.logout();
    let body = format!("<h1>logged out ID {user}</h1>");
    HttpResponse::Ok().body(body)
}
