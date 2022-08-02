use crate::db::users::{
    check_login, get_user as DBget_user, insert_user as DBinsert_user, show_users as DBshow_users,
};
use crate::models::Pool;
use actix_files::NamedFile;
use actix_identity::Identity;
use actix_session::Session;
use actix_web::http::header::LOCATION;
use actix_web::{web, Error, HttpMessage, HttpRequest, HttpResponse, Result};
use diesel::PgConnection;
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

pub async fn login_form(
    req: HttpRequest,
    form: web::Form<Login>,
    pool: web::Data<Pool>,
    id: Option<Identity>,
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

    let result = check_login(connection, &name, &pwd);
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
                session.insert("user_id", &user.id);
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

pub async fn add_user(pool: web::Data<Pool>, info: web::Form<NewUser>) -> HttpResponse {
    let connection: &mut PgConnection = &mut pool.get().unwrap();

    let name = info.username.to_string();
    let pwd = info.password.to_string();

    DBinsert_user(connection, &name, &pwd);
    HttpResponse::Ok().body(format!("username: {}", name))
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
    HttpResponse::Ok().body(format!("{:?}", response))
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct IndexResponse {
    user_id: Option<String>,
    counter: i32,
}

pub async fn show_login(id: Option<Identity>, session: Session) -> HttpResponse {
    let auth = if let Some(id) = id {
        format!("logged in: {:?}", id.id())
    } else {
        String::from("not logged in")
    };

    let username: String = session
        .get::<String>("username")
        .unwrap_or(Some("Sesson not found".to_string()))
        .unwrap_or("Sesson not found".to_string());
    let user_id: i32 = session
        .get::<i32>("user_id")
        .unwrap_or(Some(0))
        .unwrap_or(0);

    return HttpResponse::Ok().body(format!(
        "user_id {:?}  id_id {:?} username {:?}",
        user_id, auth, username
    ));
