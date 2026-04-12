use uuid::Uuid;

pub(crate) type Result<T> = core::result::Result<T, MessagePoolError>;

#[derive(thiserror::Error, Debug)]
pub enum MessagePoolError {
    #[error(transparent)]
    Config(#[from] nihility_config::ConfigError),

    #[error(transparent)]
    Store(#[from] nihility_store_operate::error::StoreError),

    #[error(transparent)]
    Uuid(#[from] uuid::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::error::Error),

    #[error("Scene not found: {0}")]
    SceneNotFound(Uuid),

    #[error("Message not found: {0}")]
    MessageNotFound(Uuid),
}
