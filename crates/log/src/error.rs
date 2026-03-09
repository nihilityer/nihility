#[derive(thiserror::Error, Debug)]
pub enum NihilityLogError {
    #[error("Config error")]
    Config(#[from] nihility_config::ConfigError),
}
