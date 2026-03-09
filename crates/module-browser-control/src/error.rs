pub(crate) type Result<T> = core::result::Result<T, BrowserControlError>;

#[derive(thiserror::Error, Debug)]
pub enum BrowserControlError {
    #[error(transparent)]
    Config(#[from] nihility_config::ConfigError),
    #[error("Build browser config failed with {0}")]
    BuildConfig(String),
    #[error(transparent)]
    Cdp(#[from] chromiumoxide::error::CdpError),
}
