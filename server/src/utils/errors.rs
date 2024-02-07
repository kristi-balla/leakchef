

use actix_http::{header::CONTENT_TYPE, StatusCode};
use actix_web::{HttpResponse, HttpResponseBuilder, ResponseError};

/// The errors that can occur when using the auth middleware.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    #[error("'X-API-Key' header is not set")]
    MissingHeader,
    #[error("Value of 'X-API-Key' header contains invalid characters")]
    InvalidFormat,
    #[error("An internal server error occurred during authentication")]
    Internal,
}

impl ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .insert_header((CONTENT_TYPE, "text/plain"))
            .body(format!("Error: {self}"))
    }
    fn status_code(&self) -> StatusCode {
        match *self {
            Self::MissingHeader => StatusCode::BAD_REQUEST,
            Self::InvalidFormat => StatusCode::BAD_REQUEST,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RoutesError {
    #[error("A Database error ocurred")]
    DatabaseClient(#[from] DatabaseError),

    #[error("Missing API KEY in the header")]
    ApiKey,

    #[error("Empty reponse")]
    ResultIsEmpty,

    #[error("Failed to perform some cryptographic operation")]
    Crypto,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum DatabaseError {
    #[error("Could not connect to database")]
    Connection,

    #[error("Couldn't find what I was looking for")]
    Find,

    #[error("Couldn't insert what I wanted to")]
    Insert,

    #[error("Couldn't update what I wanted to")]
    Update,

    #[error("Couldn't collect the result")]
    Collect,

    #[error("Result of the query is empty")]
    ResultIsEmpty,

    #[error("Something went wrong while playing with the cache")]
    Cache,
}

#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum CacheError {
    #[error("Couldn't aquire a lock for the mutex")]
    Mutex,
}
