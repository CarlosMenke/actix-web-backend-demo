use crate::db::users::{check_login, get_user as DBget_user, show_users as DBshow_users};
use crate::models::Pool;
use actix_identity::Identity;
use actix_session::Session;
use actix_web::web::Payload;
use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Result};
use diesel::PgConnection;
use log::{debug, error, info};
use serde::Deserialize;

use crate::errors::ServiceError;

use crate::models::User;

#[derive(Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub struct LoginRequest {
    username: String,
    password: String,
}
#[derive(Deserialize)]
pub struct LoginResponse {
    id: usize,
    username: String,
    token: String,
}
