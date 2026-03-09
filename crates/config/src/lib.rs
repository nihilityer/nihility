pub use crate::error::ConfigError;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use tracing::debug;

mod error;

const DEFAULT_BASE_PATH: &str = "./config";
const TOML_SUFFIX: &str = "toml";
const JSON_SUFFIX: &str = "json";

pub fn get_config<T>(module_name: &str) -> Result<T, ConfigError>
where
    T: Serialize + DeserializeOwned + Default,
{
    debug!("Loading Module {} Config", module_name);
    let base_path = option_env!("NIHILITY_CONFIG_PATH").unwrap_or_else(|| DEFAULT_BASE_PATH);
    init_base_path(base_path)?;
    let prefix = format!("{}/{}", base_path, module_name);
    if Path::try_exists(format!("{}.{}", prefix, TOML_SUFFIX).as_ref())?
        && cfg!(feature = "toml_config")
    {
        let toml_str = fs::read_to_string(format!("{}.{}", prefix, TOML_SUFFIX))?;
        return Ok(toml::from_str(toml_str.as_str())?);
    } else if Path::try_exists(format!("{}.{}", prefix, JSON_SUFFIX).as_ref())?
        && cfg!(feature = "json_config")
    {
        let reader = BufReader::new(File::open(format!("{}.{}", prefix, JSON_SUFFIX))?);
        return Ok(serde_json::from_reader(reader)?);
    } else {
        let mut config_file: File = File::create(format!("{}.{}", prefix, TOML_SUFFIX))?;
        config_file.write_all(toml::to_string_pretty(&T::default())?.as_bytes())?;
        config_file.flush()?;
    }
    Ok(T::default())
}

pub fn set_config<T>(module_name: &str, module_config: &T) -> Result<(), ConfigError>
where
    T: Serialize,
{
    debug!("Set Module {} Config", module_name);
    let base_path = option_env!("NIHILITY_CONFIG_PATH").unwrap_or_else(|| DEFAULT_BASE_PATH);
    init_base_path(base_path)?;
    let prefix = format!("{}/{}", base_path, module_name);
    if Path::try_exists(format!("{}.{}", prefix, TOML_SUFFIX).as_ref())?
        && cfg!(feature = "toml_config")
    {
        let mut config_file = File::options()
            .write(true)
            .open(format!("{}.{}", prefix, TOML_SUFFIX))?;
        config_file.write_all(toml::to_string_pretty(module_config)?.as_bytes())?;
        config_file.flush()?;
    } else if Path::try_exists(format!("{}.{}", prefix, JSON_SUFFIX).as_ref())?
        && cfg!(feature = "json_config")
    {
        let mut config_file = File::options()
            .write(true)
            .open(format!("{}.{}", prefix, JSON_SUFFIX))?;
        config_file.write_all(serde_json::to_string_pretty(module_config)?.as_bytes())?;
        config_file.flush()?;
    } else {
        let mut config_file: File = File::create(format!("{}.{}", prefix, TOML_SUFFIX))?;
        config_file.write_all(toml::to_string_pretty(module_config)?.as_bytes())?;
        config_file.flush()?;
    }

    Ok(())
}

fn init_base_path<P: AsRef<Path>>(path: P) -> Result<(), ConfigError> {
    if !Path::try_exists(path.as_ref())? {
        fs::create_dir_all(path)?;
    } else if path.as_ref().is_file() {
        fs::create_dir(path)?;
    }
    Ok(())
}
