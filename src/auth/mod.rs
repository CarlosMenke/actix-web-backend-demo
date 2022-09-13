pub mod claims;

use claims::Claims;

use actix_web::dev::ServiceRequest;
use actix_web::Error;
use actix_web_grants::permissions::AttachPermissions;
use actix_web_httpauth::extractors::bearer::BearerAuth;

use log::debug;
use serde::{Deserialize, Serialize};

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // We just get permissions from JWT
    let result = claims::decode_jwt(credentials.token());
    match result {
        Ok(claims) => {
            req.attach(claims.permissions);
            debug!("Token is valid");
            Ok(req)
        }
        // required by `actix-web-httpauth` validator signature
        Err(e) => {
            debug!("Token is invalid");
            Err((e, req))
        }
    }
}

pub async fn create_token(username: String, permissions: Vec<String>) -> Result<String, Error> {
    let claims = Claims::new(username, permissions);
    let jwt = claims::create_jwt(claims)?;
    Ok(jwt)
}

#[derive(Deserialize)]
pub struct UserPermissions {
    pub username: String,
    pub permissions: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UserPermissionsResponse {
    pub username: String,
    pub permissions: Vec<String>,
    pub token: String,
}

//TODO implement enum or so with all role models, to restirct them
