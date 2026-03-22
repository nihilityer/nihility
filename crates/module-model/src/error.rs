pub(crate) type Result<T> = core::result::Result<T, ModelError>;

#[derive(thiserror::Error, Debug)]
pub enum ModelError {
    #[error(transparent)]
    Config(#[from] nihility_config::ConfigError),

    #[error(transparent)]
    Ort(#[from] ort::Error),

    #[error(transparent)]
    ReadNpy(#[from] ndarray_npy::ReadNpyError),

    #[error(transparent)]
    ApiRequest(#[from] async_openai::error::OpenAIError),

    #[error(transparent)]
    Audio(#[from] nihility_module_audio::error::AudioError),

    #[error(transparent)]
    Wav(#[from] hound::Error),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("No available model for capability")]
    NoAvailableModel,

    #[error("All models failed: {0:?}")]
    AllModelsFailed(Vec<(String, ModelError)>),

    #[error("Unsupported operation: {0}")]
    Unsupported(String),
}
