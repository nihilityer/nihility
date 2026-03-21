pub(crate) type Result<T> = core::result::Result<T, EdgeDeviceControlError>;

#[derive(thiserror::Error, Debug)]
pub enum EdgeDeviceControlError {
    #[error(transparent)]
    Config(#[from] nihility_config::ConfigError),

    #[error(transparent)]
    Uuid(#[from] uuid::Error),

    #[error(transparent)]
    Mdns(#[from] mdns_sd::Error),

    #[error(transparent)]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error(transparent)]
    Image(#[from] image::ImageError),

    #[error(transparent)]
    BrowserControl(#[from] nihility_module_browser_control::error::BrowserControlError),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Device status error: {0}")]
    DeviceStatus(String),

    #[error("Module status error: {0}")]
    ModuleStatus(String),
}
