use crate::db::users::get_user as DBget_user;
use crate::models::Pool;
use actix_web::guard::Header;
use actix_web::web::Payload;
use actix_web::{web, HttpRequest, Result};
use actix_web_grants::proc_macro::{has_any_role, has_permissions};
use diesel::PgConnection;
use futures::StreamExt;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;

use crate::errors::ServiceError;

use crate::auth::{create_token, UserPermissions, UserPermissionsResponse};
use crate::handlers::pages::NewUser;
use crate::models::User;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResponseHtml {
    pub html: String,
}

pub async fn test_html() -> Result<web::Json<ResponseHtml>, ServiceError> {
    let file = fs::read_to_string("./files/music_all.html").expect("Unable to read file");
    Ok(web::Json(ResponseHtml { html: file }))
}

pub async fn test_get(
    pool: web::Data<Pool>,
) -> Result<web::Json<SendMessageResponseBody>, ServiceError> {
    let connection: &mut PgConnection = &mut pool.get().unwrap();

    Ok(web::Json(SendMessageResponseBody {
        ordinal_number: 42,
        text: "response".to_owned(),
    }))
}

pub async fn test_get_vec(
    pool: web::Data<Pool>,
) -> Result<web::Json<SendMessageResponseBodyVec>, ServiceError> {
    let connection: &mut PgConnection = &mut pool.get().unwrap();

    let mut response_vec = Vec::new();
    response_vec.push(SendMessageResponseBody {
        ordinal_number: 42,
        text: "response".to_owned(),
    });
    response_vec.push(SendMessageResponseBody {
        ordinal_number: 22,
        text: "response2".to_owned(),
    });

    Ok(web::Json(SendMessageResponseBodyVec {
        response: response_vec,
    }))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SendMessageRequestBody {
    pub text: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SendMessageResponseBody {
    pub ordinal_number: u32,
    pub text: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SendMessageResponseBodyVec {
    pub response: Vec<SendMessageResponseBody>,
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k
pub async fn test_post(
    pool: web::Data<Pool>,
    mut payload: Payload,
) -> Result<web::Json<SendMessageResponseBody>, ServiceError> {
    let connection: &mut PgConnection = &mut pool.get().unwrap();

    // payload is a stream of Bytes objects
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        //TODO remove unwrap
        let chunk = chunk.unwrap();
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(ServiceError::BadRequest("overflow".to_string()));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let obj = serde_json::from_slice::<SendMessageRequestBody>(&body)?;

    debug!("test_post method called! {:?}", obj);
    Ok(web::Json(SendMessageResponseBody {
        ordinal_number: 32,
        text: obj.text,
    }))
}

pub async fn get_user(
    pool: web::Data<Pool>,
    form: web::Form<NewUser>,
) -> Result<web::Json<User>, ServiceError> {
    let connection: &mut PgConnection = &mut pool.get().unwrap();

    let name = form.username.to_string();
    let pwd = form.password.to_string();

    let result = DBget_user(connection, &name, &pwd);
    match result {
        Ok(u) => Ok(web::Json(u)),
        Err(_) => Err(ServiceError::InternalServerError(
            "User not found".to_string(),
        )),
    }
}

// Tests for authentithication via JWT
// it will generate a valid token, for logging in later in session
// every user will get a login for all test functions, how requests login
//
// An additional handler for generating a token.
// Thus, you can try to create your own tokens and check the operation of the permissions system.
// cURL example:
// ```sh
//  curl --location --request POST 'http://localhost:8080/token' \
//   --header 'Content-Type: application/json' \
//   --data-raw '{
//       "username": "Lorem-Ipsum",
//       "permissions": ["OP_GET_SECURED_INFO"]
//   }'
// ```
pub async fn test_login(
    mut payload: Payload,
    //) -> Result<web::Json<UserPermissionsResponse>, ServiceError> {
) -> Result<String, ServiceError> {
    debug!("test_login function called");
    let mut body = web::BytesMut::new();
    while let Some(chunk) = payload.next().await {
        //TODO remove unwrap
        let chunk = chunk.unwrap();
        // limit max size of in-memory payload
        if (body.len() + chunk.len()) > MAX_SIZE {
            return Err(ServiceError::BadRequest("overflow".to_string()));
        }
        body.extend_from_slice(&chunk);
    }

    // body is loaded, now we can deserialize serde-json
    let pay = serde_json::from_slice::<UserPermissions>(&body)?;
    let token_str = match create_token(pay.username.clone(), pay.permissions.clone()).await {
        Ok(t) => t,
        Err(_) => {
            return Err(ServiceError::InternalServerError(
                "Failed to create login Token".to_string(),
            ))
        }
    };

    let response = UserPermissionsResponse {
        username: pay.username,
        permissions: pay.permissions,
        token: token_str.clone(),
    };
    Ok(token_str)
}

#[has_any_role("ADMIN")]
// You can check via cURL (for generate you own token, use `/test/login` handler):
// ```sh
//  curl --location --request GET 'http://localhost:8080/api/manager' \
//  --header 'Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VybmFtZSI6IkxvcmVtLUlwc3VtIiwicGVybWlzc2lvbnMiOlsiUk9MRV9NQU5BR0VSIl0sImV4cCI6MTkyNjY5MDYxN30.AihInANG_8gp5IZk5gak9-Sv_ankb740FfNepzhZojw'
// ```
pub async fn test_admin_page() -> Result<web::Json<SendMessageResponseBody>, ServiceError> {
    debug!("test_admin function called");
    Ok(web::Json(SendMessageResponseBody {
        ordinal_number: 42,
        text: "response".to_owned(),
    }))
}
