use axum::Json;
use axum::http::StatusCode;
use thiserror::Error;

use crate::api::ApiError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Not found")]
    NotFound,

    #[error("Abort")]
    Abort,

    #[error("Invalid PIN")]
    InvalidPin,

    #[error("Counter error")]
    Counter,

    #[error("Empty update")]
    EmptyUpdate,

    #[error(transparent)]
    NanoPulse(#[from] nanopulse::errors::Error),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    Infallible(#[from] std::convert::Infallible),

    #[error(transparent)]
    TryFromInt(#[from] std::num::TryFromIntError),

    #[error(transparent)]
    HeaplessCapacity(#[from] heapless::CapacityError),

    #[error(transparent)]
    Validation(#[from] validator::ValidationErrors),
}

impl Error {
    pub fn from_sqlx(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::RowNotFound => Error::NotFound,
            _ => Error::from(e),
        }
    }

    pub fn status_code(&self) -> StatusCode {
        match self {
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::Abort => StatusCode::INTERNAL_SERVER_ERROR,
            Error::InvalidPin => StatusCode::UNPROCESSABLE_ENTITY,
            Error::NanoPulse(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::Sqlx(sqlx::Error::RowNotFound) => StatusCode::NOT_FOUND,
            Error::Sqlx(sqlx::Error::Database(_)) => StatusCode::UNPROCESSABLE_ENTITY,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn description(&self) -> String {
        match self {
            Error::Sqlx(sqlx::Error::Database(v)) => {
                let v = v.to_string();
                if v.contains("UNIQUE constraint") {
                    "Object already exist by the same identifier".into()
                } else {
                    v
                }
            }
            _ => self.to_string(),
        }
    }
}

impl From<Error> for (StatusCode, Json<ApiError>) {
    fn from(value: Error) -> Self {
        (
            value.status_code(),
            Json(ApiError {
                message: value.description(),
            }),
        )
    }
}
