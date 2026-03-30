use axum::http;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tracing::error;

pub(crate) type Result<T> = core::result::Result<T, NihilityServerError>;

#[derive(thiserror::Error, Debug)]
pub enum NihilityServerError {
    #[error("Resource Not Found: {0}")]
    NotFound(String),
    #[error("Invalid Token")]
    InvalidToken,
    #[error("Missing Credentials")]
    MissingCredentials,
    #[error("Wrong Credentials")]
    WrongCredentials,
    #[error("Invalid Config: {0}")]
    Config(String),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Db(#[from] sea_orm::DbErr),
    #[error(transparent)]
    Http(#[from] http::Error),
    #[error("Build password hash error: {0}")]
    PasswordHash(argon2::password_hash::Error),
    #[error("JWT encoding error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error(transparent)]
    InvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    #[error(transparent)]
    ModuleManager(#[from] nihility_module_manager::error::ModuleManagerError),
    #[error(transparent)]
    ConfigError(#[from] nihility_config::ConfigError),
}

impl IntoResponse for NihilityServerError {
    fn into_response(self) -> Response {
        match self {
            NihilityServerError::NotFound(name) => {
                let err_msg = format!("Resource Not Found: {}", name);
                error!("{}", err_msg);
                (StatusCode::NOT_FOUND, err_msg)
            }
            NihilityServerError::InvalidToken => {
                (StatusCode::UNAUTHORIZED, "Invalid token".to_string())
            }
            NihilityServerError::MissingCredentials => {
                (StatusCode::BAD_REQUEST, "Missing Credentials".to_string())
            }
            NihilityServerError::WrongCredentials => {
                (StatusCode::UNAUTHORIZED, "Wrong credentials".to_string())
            }
            NihilityServerError::Config(desc) => {
                error!("Invalid config: {}", desc);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Invalid Config".to_string(),
                )
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
            NihilityServerError::PasswordHash(e) => {
                error!("Build Password Hash Error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Build Password Hash Error".to_string(),
                )
            }
            NihilityServerError::Jwt(e) => {
                error!("JWT error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Jwt Error".to_string())
            }
            NihilityServerError::InvalidHeaderValue(e) => {
                error!("Invalid Header Value: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Invalid Header Value".to_string(),
                )
            }
            NihilityServerError::ModuleManager(e) => {
                error!("Module Manager Error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Module Manager Error".to_string(),
                )
            }
            NihilityServerError::ConfigError(e) => {
                error!("Config Error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Config Error".to_string(),
                )
            }
        }
        .into_response()
    }
}
