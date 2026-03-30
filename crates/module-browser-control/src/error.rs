pub(crate) type Result<T> = core::result::Result<T, BrowserControlError>;

#[derive(thiserror::Error, Debug)]
pub enum BrowserControlError {
    #[error(transparent)]
    Config(#[from] nihility_config::ConfigError),
    #[error("Build browser config failed with {0}")]
    BuildConfig(String),
    #[error("Download browser failed with {0}")]
    Download(#[from] chromiumoxide::fetcher::FetcherError),
    #[error("Operation error: {0}")]
    Operation(String),
    #[error(transparent)]
    Cdp(#[from] chromiumoxide::error::CdpError),
    #[error("Build execute parameters failed with {0}")]
    ExecuteParam(String),
    #[error(transparent)]
    Uuid(#[from] uuid::Error),
}
