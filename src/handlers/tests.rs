use crate::db::users::get_user as DBget_user;
use crate::models::Pool;
use actix_web::web::Payload;
use actix_web::{web, Result};
use diesel::PgConnection;
use futures::StreamExt;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::errors::ServiceError;

use crate::handlers::pages::NewUser;
use crate::models::User;

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
// ------ ------
//     Init
// ------ ------

// ------ ------
//     Init
// ------ ------

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
        text: "response".to_owned(),
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
