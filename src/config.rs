use crate::log::LogConfig;
use anyhow::Result;
use figment::Figment;
use figment::providers::{Format, Json, Serialized, Toml, Yaml};
#[cfg(feature = "chat-bot")]
use nihility_input_chat::WsConfig;
#[cfg(feature = "simple-memory")]
use nihility_memory_simple::NihilitySimpleMemoryConfig;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

const JSON_CONFIG_FILE_NAME: &str = "config.json";
const TOML_CONFIG_FILE_NAME: &str = "config.toml";
const YAML_CONFIG_FILE_NAME: &str = "config.yaml";

#[derive(Deserialize, Serialize, Debug)]
pub struct NihilityConfig {
    pub log: Vec<LogConfig>,
    #[cfg(feature = "chat-bot")]
    pub chat_bot: WsConfig,
    #[cfg(feature = "simple-memory")]
    pub simple_memory: NihilitySimpleMemoryConfig,
}

impl Default for NihilityConfig {
    fn default() -> Self {
        Self {
            log: vec![LogConfig::default()],
            #[cfg(feature = "chat-bot")]
            chat_bot: WsConfig::default(),
            #[cfg(feature = "simple-memory")]
            simple_memory: NihilitySimpleMemoryConfig {
                embedding_model: "text-embedding-v3".to_string(),
                openie_model: "qwen-plus".to_string(),
                api_base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
                api_key: "".to_string(),
            },
        }
    }
}

impl NihilityConfig {
    pub fn init() -> Result<Self> {
        let config = NihilityConfig::default();
        if Path::try_exists(TOML_CONFIG_FILE_NAME.as_ref())? {
            Ok(Figment::merge(
                Figment::from(Serialized::defaults(config)),
                Toml::file(TOML_CONFIG_FILE_NAME),
            )
            .extract()?)
        } else if Path::try_exists(YAML_CONFIG_FILE_NAME.as_ref())? {
            Ok(Figment::from(Serialized::defaults(config))
                .merge(Yaml::file(YAML_CONFIG_FILE_NAME))
                .extract()?)
        } else if Path::try_exists(JSON_CONFIG_FILE_NAME.as_ref())? {
            Ok(Figment::from(Serialized::defaults(config))
                .merge(Json::file(JSON_CONFIG_FILE_NAME))
                .extract()?)
        } else {
            let mut config_file: File = File::create(TOML_CONFIG_FILE_NAME)?;
            config_file.write_all(toml::to_string_pretty(&config)?.as_bytes())?;
            config_file.flush()?;
            Ok(config)
        }
    }
}
