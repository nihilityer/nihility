pub use crate::error::ConfigError;
use schemars::JsonSchema;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use tracing::debug;
use uuid::Uuid;

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

#[cfg(feature = "db")]
pub async fn get_config_with_db<T>(
    module_name: &str,
    conn: &DatabaseConnection,
) -> Result<T, ConfigError>
where
    T: Serialize + DeserializeOwned + Default + JsonSchema,
{
    if check_config_exists(module_name)? {
        return get_config(module_name);
    }
    let entity_result = nihility_server_entity::module_config::Entity::find()
        .filter(nihility_server_entity::module_config::Column::ModuleName.eq(module_name))
        .one(conn)
        .await?;
    if let Some(record) = entity_result {
        let config: T = serde_json::from_value(record.config_value)
            .map_err(|e| ConfigError::InvalidConfig(e.to_string()))?;
        debug!("Loaded config from database for module: {}", module_name);
        return Ok(config);
    }
    let default_config = T::default();
    set_config_with_db(module_name, &default_config, conn).await?;
    debug!(
        "Created default config in database for module: {}",
        module_name
    );
    Ok(default_config)
}

#[cfg(feature = "db")]
pub async fn set_config_with_db<T>(
    module_name: &str,
    config: &T,
    conn: &DatabaseConnection,
) -> Result<(), ConfigError>
where
    T: Serialize + JsonSchema,
{
    let json_value =
        serde_json::to_value(config).map_err(|e| ConfigError::InvalidConfig(e.to_string()))?;
    let json_schema = serde_json::to_value(&schemars::schema_for!(T))
        .map_err(|e| ConfigError::SchemaGen(e.to_string()))?;
    let now = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
    let existing = nihility_server_entity::module_config::Entity::find()
        .filter(nihility_server_entity::module_config::Column::ModuleName.eq(module_name))
        .one(conn)
        .await?;
    if let Some(record) = existing {
        // Update existing
        let mut active_model: nihility_server_entity::module_config::ActiveModel = record.into();
        active_model.config_value = Set(json_value);
        active_model.json_schema = Set(json_schema);
        active_model.updated_at = Set(now);
        active_model.update(conn).await?;
    } else {
        // Insert new
        let active_model = nihility_server_entity::module_config::ActiveModel {
            id: Set(Uuid::new_v4()),
            module_name: Set(module_name.to_string()),
            config_value: Set(json_value),
            json_schema: Set(json_schema),
            created_at: Set(now),
            updated_at: Set(now),
        };
        active_model.insert(conn).await?;
    }

    debug!("Saved config to database for module: {}", module_name);
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

fn check_config_exists(module_name: &str) -> Result<bool, ConfigError> {
    let base_path = option_env!("NIHILITY_CONFIG_PATH").unwrap_or_else(|| DEFAULT_BASE_PATH);
    let prefix = format!("{}/{}", base_path, module_name);
    Ok(
        Path::try_exists(format!("{}.{}", prefix, TOML_SUFFIX).as_ref())?
            || Path::try_exists(format!("{}.{}", prefix, JSON_SUFFIX).as_ref())?,
    )
}
