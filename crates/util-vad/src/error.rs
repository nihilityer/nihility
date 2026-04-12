pub(crate) type Result<T> = core::result::Result<T, VoiceActivityDetectionError>;

#[derive(thiserror::Error, Debug)]
pub enum VoiceActivityDetectionError {
    #[error(transparent)]
    Ort(#[from] ort::Error),
    #[error("Ort Session build error: {0}")]
    OrtSessionBuilder(String),
    #[error(transparent)]
    ParamShape(#[from] ndarray::ShapeError),
    #[error(transparent)]
    SpeechSend(#[from] tokio::sync::broadcast::error::SendError<Vec<f32>>),
    #[error("Voice Activity Detection initialization error: {0}")]
    Init(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}
