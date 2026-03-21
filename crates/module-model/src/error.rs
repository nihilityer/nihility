pub(crate) type Result<T> = core::result::Result<T, ModelError>;

#[derive(thiserror::Error, Debug)]
pub enum ModelError {
    #[error(transparent)]
    Config(#[from] nihility_config::ConfigError),

    #[error(transparent)]
    ApiRequest(#[from] async_openai::error::OpenAIError),

    #[error(transparent)]
    Audio(#[from] hound::Error),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("No available model for capability")]
    NoAvailableModel,

    #[error("All models failed")]
    AllModelsFailed,

    #[error("Unsupported operation: {0}")]
    Unsupported(String),

    #[error("Audio encode error: {0}")]
    AudioEncode(String),
}
