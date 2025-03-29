use anyhow::{Result, anyhow};
use config::Config;
use nihility_common::config::{NihilityConfig, set_config};
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

const TOML_SUFFIX: &str = "toml";
const JSON_SUFFIX: &str = "json";
const YAML_SUFFIX: &str = "yaml";

pub struct NihilityConfigPlugin {
    base_path: String,
}

impl NihilityConfig for NihilityConfigPlugin {
    fn get_config(&self, plugin_name: String, default: Value) -> Result<Value> {
        if !Path::try_exists(self.base_path.as_ref())? {
            let dir = Path::new(&self.base_path);
            std::fs::create_dir_all(dir)?;
        } else {
            let dir = Path::new(&self.base_path);
            if dir.is_file() {
                return Err(anyhow!("Base path must a directory"));
            }
        }
        let prefix = format!("{}/{}", self.base_path, plugin_name);
        if Path::try_exists(format!("{}.{}", prefix, TOML_SUFFIX).as_ref())?
            || Path::try_exists(format!("{}.{}", prefix, JSON_SUFFIX).as_ref())?
            || Path::try_exists(format!("{}.{}", prefix, YAML_SUFFIX).as_ref())?
        {
            let config_value = Config::builder()
                .add_source(config::File::with_name(prefix.as_str()))
                .build()?
                .try_deserialize()?;
            Ok(config_value)
        } else {
            let file_value = serde_json::to_string_pretty(&default)?;
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(format!("{}.{}", prefix, JSON_SUFFIX))?;
            file.write_all(file_value.as_bytes())?;
            file.sync_all()?;
            Ok(default)
        }
    }
}

impl NihilityConfigPlugin {
    pub async fn init<T: Into<String>>(base_path: T) -> Result<()> {
        set_config(Box::new(Self {
            base_path: base_path.into(),
        }))
        .await;
        Ok(())
    }
}
