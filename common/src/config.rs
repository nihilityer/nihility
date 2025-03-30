use crate::CONFIG;
use anyhow::{Result, anyhow};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

const GLOBAL_CONFIG_FILE: &str = "global";

pub trait NihilityConfig: Send + Sync {
    fn get_config(&self, plugin_name: String, default: Value) -> Result<Value>;
}

pub async fn set_config(config: Box<dyn NihilityConfig + Send + Sync>) {
    CONFIG.lock().await.replace(config);
}

pub async fn get_config<C>(plugin_name: String) -> Result<C>
where
    C: DeserializeOwned + Clone + Default + Serialize,
{
    match CONFIG.lock().await.as_ref() {
        None => Err(anyhow!("Config not initialized")),
        Some(config_core) => {
            let default_value: Value = serde_json::to_value(C::default())?;
            let config = config_core.get_config(plugin_name, default_value)?;
            serde_json::from_value::<C>(config.clone()).map_err(|e| anyhow!(e))
        }
    }
}

pub async fn get_global_config(key: &str, default: Value) -> Result<Value> {
    match CONFIG.lock().await.as_ref() {
        None => Err(anyhow!("Config not initialized")),
        Some(config_core) => {
            let default_value: Value = Value::Object(serde_json::Map::new());
            let config = config_core.get_config(GLOBAL_CONFIG_FILE.to_string(), default_value)?;
            if let Some(value) = config.get(key) {
                Ok(value.clone())
            } else {
                Ok(default)
            }
        }
    }
}
