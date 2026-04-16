pub(crate) type Result<T> = core::result::Result<T, SceneError>;

#[derive(thiserror::Error, Debug)]
pub enum SceneError {
    #[error(transparent)]
    Config(#[from] nihility_config::ConfigError),

    #[error(transparent)]
    Database(#[from] nihility_store_operate::StoreError),

    #[error(transparent)]
    Uuid(#[from] uuid::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
