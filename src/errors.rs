use actix_session::SessionInsertError;
use actix_web::{error::ResponseError, HttpResponse};
use anyhow;
use argon2::password_hash::Error as ArgonError;
use derive_more::Display;
use diesel::result::{DatabaseErrorKind, Error as DBError};
use serde_json::Error as SerdeJsonError;

#[derive(Debug, Display)]
pub enum ServiceError {
    #[display(fmt = "Internal Server Error: {}", _0)]
    InternalServerError(String),

    #[display(fmt = "BadRequest: {}", _0)]
    BadRequest(String),

    #[display(fmt = "Unauthorized")]
    Unauthorized,
}

// impl ResponseError trait allows to convert our errors into http responses with appropriate data
impl ResponseError for ServiceError {
    fn error_response(&self) -> HttpResponse {
        match self {
            ServiceError::InternalServerError(ref message) => {
                HttpResponse::InternalServerError().json(message)
            }
            ServiceError::BadRequest(ref message) => HttpResponse::BadRequest().json(message),
            ServiceError::Unauthorized => HttpResponse::Unauthorized().json("Unauthorized"),
        }
    }
}

impl From<DBError> for ServiceError {
    fn from(error: DBError) -> ServiceError {
        // Right now we just care about UniqueViolation from diesel
        match error {
            DBError::DatabaseError(kind, info) => {
                if let DatabaseErrorKind::UniqueViolation = kind {
                    let message = info.details().unwrap_or_else(|| info.message()).to_string();
                    return ServiceError::BadRequest(message);
                }
                ServiceError::InternalServerError("DBError from diesel".to_string())
            }
            _ => ServiceError::InternalServerError("DBError from diesel".to_string()),
        }
    }
}

impl From<ArgonError> for ServiceError {
    fn from(error: ArgonError) -> ServiceError {
        match error {
            _ => ServiceError::InternalServerError("ArgonError from Argon".to_string()),
        }
    }
}

impl From<SerdeJsonError> for ServiceError {
    fn from(error: SerdeJsonError) -> ServiceError {
        match error {
            _ => {
                ServiceError::InternalServerError("Error for converting serde to json".to_string())
            }
        }
    }
}

impl From<anyhow::Error> for ServiceError {
    fn from(error: anyhow::Error) -> ServiceError {
        match error {
            _ => ServiceError::InternalServerError("Anyhow Error".to_string()),
        }
    }
}

impl From<SessionInsertError> for ServiceError {
    fn from(error: SessionInsertError) -> ServiceError {
        match error {
            _ => ServiceError::InternalServerError("Session inssert Error".to_string()),
        }
    }
}
