use tokio::task::JoinError;

pub(crate) type Result<T> = core::result::Result<T, InitError>;

#[derive(thiserror::Error, Debug)]
pub enum InitError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    HfApi(#[from] hf_hub::api::tokio::ApiError),
    #[error("Model install error: {0}")]
    Model(String),
    #[error("Chromium install error: {0}")]
    Chromium(String),
    #[error("System dependencies install error: {0}")]
    DepsInstall(String),
    #[error("{0}")]
    Unknown(String),
    #[error(transparent)]
    Task(JoinError),
}
