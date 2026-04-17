pub(crate) type Result<T> = core::result::Result<T, EdgeDeviceControlError>;

#[derive(thiserror::Error, Debug)]
pub enum EdgeDeviceControlError {
    #[error(transparent)]
    Config(#[from] nihility_config::ConfigError),

    #[error(transparent)]
    Uuid(#[from] uuid::Error),

    #[error(transparent)]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error(transparent)]
    Image(#[from] image::ImageError),

    #[error(transparent)]
    BrowserControl(#[from] nihility_module_browser_control::error::BrowserControlError),

    #[error(transparent)]
    Model(#[from] nihility_module_model::error::ModelError),

    #[error(transparent)]
    MessagePool(#[from] nihility_module_message_pool::error::MessagePoolError),

    #[error(transparent)]
    Vad(#[from] nihility_util_vad::error::VoiceActivityDetectionError),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Device status error: {0}")]
    DeviceStatus(String),

    #[error("Module status error: {0}")]
    ModuleStatus(String),

    #[error("Other error: {0}")]
    Other(String),
}
