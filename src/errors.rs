use actix_web::{error::JsonPayloadError, http::StatusCode, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use wither::{
    bson::{oid::Error as ObjectIDError, ser::Error as BSONEncodingError},
    WitherError,
};

#[derive(Debug, Deserialize, Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug, Error)]
pub enum TodoErrors {
    #[error("Internal server error")]
    DatabaseError(#[from] WitherError),
    #[error("Invalid document")]
    BSONEncodingError(#[from] BSONEncodingError),
    #[error("Todo with id {0} could not be found")]
    TodoNotFound(String),
    #[error("Invalid ID")]
    InvalidID(#[from] ObjectIDError),
    #[error("{0}")]
    InvalidJsonBody(#[from] JsonPayloadError),
}

impl ResponseError for TodoErrors {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorResponse {
            error: self.to_string(),
        })
    }

    fn status_code(&self) -> StatusCode {
        match self {
            TodoErrors::TodoNotFound(_) => StatusCode::NOT_FOUND,
            TodoErrors::BSONEncodingError(_) => StatusCode::BAD_REQUEST,
            TodoErrors::InvalidID(_) => StatusCode::BAD_REQUEST,
            TodoErrors::InvalidJsonBody(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
