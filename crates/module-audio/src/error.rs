use thiserror::Error;

pub(crate) type Result<T> = core::result::Result<T, AudioError>;

#[derive(Error, Debug)]
pub enum AudioError {
    #[error(transparent)]
    Config(#[from] nihility_config::ConfigError),

    #[error(transparent)]
    Wav(#[from] hound::Error),

    #[error(transparent)]
    NdArray(#[from] ndarray::ShapeError),

    #[error(transparent)]
    Ort(#[from] ort::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),

    #[error("Unsupported bits_per_sample: {0}")]
    UnsupportedBitsPerSample(u8),

    #[error("Unsupported sample rate: {0}")]
    UnsupportedSampleRate(u32),

    #[error("Invalid channel count: {0}, supported values are 1 or 2")]
    InvalidChannelCount(u8),

    #[error("Invalid WAV data: {0}")]
    InvalidWavData(String),

    #[error("PCM data length mismatch: expected {expected}, got {actual}")]
    PcmLengthMismatch { expected: usize, actual: usize },

    #[error("VAD error: {0}")]
    VadError(String),

    #[error("VAD init error: {0}")]
    VadInitError(String),
}