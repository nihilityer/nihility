pub type Result<T> = core::result::Result<T, ModuleManagerError>;

#[derive(thiserror::Error, Debug)]
pub enum ModuleManagerError {
    #[error(transparent)]
    Config(#[from] nihility_config::ConfigError),

    #[error(transparent)]
    BrowserControl(#[from] nihility_module_browser_control::error::BrowserControlError),

    #[error(transparent)]
    EdgeDeviceControl(#[from] nihility_module_edge_device_control::error::EdgeDeviceControlError),

    #[error(transparent)]
    Model(#[from] nihility_module_model::error::ModelError),

    #[error("Module not found: {0:?}")]
    ModuleNotFound(crate::ModuleType),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}
