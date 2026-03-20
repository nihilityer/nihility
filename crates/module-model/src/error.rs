pub(crate) type Result<T> = core::result::Result<T, ModelError>;

#[derive(thiserror::Error, Debug)]
pub enum ModelError {
    #[error(transparent)]
    Config(#[from] nihility_config::ConfigError),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("API request failed: {0}")]
    ApiRequest(String),

    #[error("API response parse error: {0}")]
    ApiParse(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("No available model for capability")]
    NoAvailableModel,

    #[error("All models failed")]
    AllModelsFailed,

    #[error("Streaming error: {0}")]
    Streaming(String),

    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    #[error("Audio decode error: {0}")]
    AudioDecode(String),
}
