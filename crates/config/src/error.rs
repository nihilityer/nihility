#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
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
    #[cfg(feature = "db")]
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),
    #[cfg(feature = "db")]
    #[error("Config not found in database for module: {0}")]
    NotFound(String),
    #[cfg(feature = "db")]
    #[error("Invalid config value: {0}")]
    InvalidConfig(String),
    #[cfg(feature = "db")]
    #[error("JsonSchema generation failed: {0}")]
    SchemaGen(String),
}
