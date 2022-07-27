use crate::db::establish_connection;
use crate::db::users::{check_login, insert_user as DBinsert_user, show_users as DBshow_users};
use actix_files::NamedFile;
use actix_web::cookie::Cookie;
use actix_web::http::header::LOCATION;
use actix_web::{web, HttpRequest, HttpResponse, Result};
use log::info;
use serde::Deserialize;
use std::path::PathBuf;

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

pub async fn login_form(form: web::Form<Login>) -> HttpResponse {
    let name = form.username.to_string();
    let pwd = form.password.to_string();
    info!(
        "Login Attempt: \t\t\tUsr: {:?}\t\tPassword: {:?}",
        name, pwd
    );

    let connection = &mut establish_connection();

    let result = check_login(connection, &name, &pwd);
    if result == false {
        return HttpResponse::Ok().body(format!("username: {}", &name));
        //HttpResponse::Ok()
        //.content_type(ContentType::html())
        //.body(include_str!("../../files/login.html"))
    } else {
        return HttpResponse::SeeOther()
            .insert_header((LOCATION, "/"))
            .cookie(Cookie::new("_flash", name.to_string()))
            .finish();
    }
}

pub async fn add_user(info: web::Form<NewUser>) -> HttpResponse {
    let connection = &mut establish_connection();

    let name = info.username.to_string();
    let pwd = info.password.to_string();

    DBinsert_user(connection, &name, &pwd);
    HttpResponse::Ok().body(format!("username: {}", name))
}

pub async fn show_users(_rew: HttpRequest) -> HttpResponse {
    let connection = &mut establish_connection();

    let result = DBshow_users(connection);
    let mut response = String::new();

    for user in result {
        response.push_str(&format!(
            "User id '{}' with Name '{}' and Pwd '{}'",
            user.id, user.username, user.password,
        ));
    }
    HttpResponse::Ok().body(format!("{:?}", response))
}
