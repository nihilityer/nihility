use axum::http;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tracing::error;

pub(crate) type Result<T> = core::result::Result<T, NihilityServerError>;

#[derive(thiserror::Error, Debug)]
pub enum NihilityServerError {
    #[error("Resource Not Found: {0}")]
    NotFound(String),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Db(#[from] sea_orm::DbErr),
    #[error(transparent)]
    Http(#[from] http::Error),
}

impl IntoResponse for NihilityServerError {
    fn into_response(self) -> Response {
        match self {
            NihilityServerError::NotFound(name) => {
                let err_msg = format!("Resource Not Found: {}", name);
                error!("{}", err_msg);
                (StatusCode::NOT_FOUND, err_msg)
            }
            NihilityServerError::IO(e) => {
                error!("Internal server error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server I/O Error".to_string(),
                )
            }
            NihilityServerError::Db(e) => {
                error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database Error".to_string(),
                )
            }
            NihilityServerError::Http(e) => {
                error!("Http error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Http Error".to_string())
            }
        }
        .into_response()
    }
}
