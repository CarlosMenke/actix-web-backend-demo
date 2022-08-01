use crate::db::establish_connection;
use crate::db::users::{
    check_login, get_user_id as DBget_user_id, insert_user as DBinsert_user,
    show_users as DBshow_users,
};
use actix_files::NamedFile;
use actix_session::Session;
use actix_web::http::header::LOCATION;
use actix_web::{error, web, HttpRequest, HttpResponse, Result};
use log::{error, info};
use serde::Deserialize;
use serde::Serialize;
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

pub async fn login_form(session: Session, form: web::Form<Login>) -> HttpResponse {
    let c: Option<String> = session.get::<String>("user_id").unwrap();

    let name = form.username.to_string();
    let pwd = form.password.to_string();
    info!(
        "Login Attempt: \t\t\tUsr: {:?}\t\tPassword: {:?}",
        name, pwd
    );

    let connection = &mut establish_connection();

    let result = check_login(connection, &name, &pwd);
    if result == false {
        return HttpResponse::Ok().body(format!("username: {} counter {:?}", &name, c));
        //HttpResponse::Ok()
        //.content_type(ContentType::html())
        //.body(include_str!("../../files/login.html"))
    } else {
        match DBget_user_id(connection, &name, &pwd) {
            Ok(id) => {
                session.insert("user_id", &id);
                session.insert("username", &name);
                session.renew();
            }
            Err(_) => error!("User with name {:?} not found", &name),
        }
        return HttpResponse::SeeOther()
            .insert_header((LOCATION, "/show_login"))
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct IndexResponse {
    user_id: Option<String>,
    counter: i32,
}

pub async fn login_custom(session: Session) -> HttpResponse {
    let id = "5";
    session.insert("user_id", &id);
    session.renew();

    //let counter: i32 = session
    //.get::<i32>("counter")
    //.unwrap_or(Some(0))
    //.unwrap_or(0);

    //Ok(HttpResponse::Ok().json(IndexResponse {
    //user_id: Some(id.to_string()),
    //counter,
    //}))
    HttpResponse::Ok().body(format!("user_id {:?} counter {:?}", id, id))
}

pub async fn show_login(session: Session) -> HttpResponse {
    let username: String = session
        .get::<String>("username")
        .unwrap_or(Some("Sesson not found".to_string()))
        .unwrap_or("Sesson not found".to_string());
    let user_id: i32 = session
        .get::<i32>("user_id")
        .unwrap_or(Some(0))
        .unwrap_or(0);

    HttpResponse::Ok().body(format!("user_id {:?} username {:?}", user_id, username))
}
