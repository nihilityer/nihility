pub(crate) type Result<T> = core::result::Result<T, MessagePoolError>;

#[derive(thiserror::Error, Debug)]
pub enum MessagePoolError {
    #[error(transparent)]
    Config(#[from] nihility_config::ConfigError),

    #[error(transparent)]
    Db(#[from] sea_orm::DbErr),

    #[error(transparent)]
    Uuid(#[from] uuid::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::error::Error),

    // Business logic errors (custom descriptions)
    #[error("Scene not found: {0}")]
    SceneNotFound(String),

    #[error("Message not found: {0}")]
    MessageNotFound(String),
}
