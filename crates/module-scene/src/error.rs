pub(crate) type Result<T> = core::result::Result<T, SceneError>;

#[derive(thiserror::Error, Debug)]
pub enum SceneError {
    #[error(transparent)]
    Config(#[from] nihility_config::ConfigError),

    #[error("Operation error: {0}")]
    Operation(String),

    #[error(transparent)]
    Database(#[from] nihility_store_operate::StoreError),

    #[error(transparent)]
    Uuid(#[from] uuid::Error),

    #[error("Scene not found: {0}")]
    NotFound(String),
}
