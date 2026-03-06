use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tracing::error;

pub(crate) type Result<T> = core::result::Result<T, NihilityServerError>;

#[derive(thiserror::Error, Debug)]
pub enum NihilityServerError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Resource Not Found: {0}")]
    NotFound(String),
}

impl IntoResponse for NihilityServerError {
    fn into_response(self) -> Response {
        match self {
            NihilityServerError::IO(e) => {
                error!("Internal server error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal Server I/O Error".to_string(),
                )
            }
            NihilityServerError::NotFound(name) => {
                let err_msg = format!("Resource Not Found: {}", name);
                error!("{}", err_msg);
                (StatusCode::NOT_FOUND, err_msg)
            }
        }
        .into_response()
    }
}
