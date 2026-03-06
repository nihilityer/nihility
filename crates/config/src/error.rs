#[derive(thiserror::Error, Debug)]
pub enum NihilityConfigError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[cfg(feature = "json_config")]
    #[error("JSON error: {0}")]
    JSON(#[from] serde_json::Error),
    #[cfg(feature = "toml_config")]
    #[error("Toml Deserialize error: {0}")]
    TomlDe(#[from] toml::de::Error),
    #[cfg(feature = "toml_config")]
    #[error("TOML Serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),
}
