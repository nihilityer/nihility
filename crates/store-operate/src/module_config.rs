use crate::StoreError;
use chrono::Utc;
use nihility_store_entity::module_config;
use nihility_store_entity::prelude::ModuleConfig;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, Set};
use uuid::Uuid;

pub async fn find_by_module_name(
    db: &DbConn,
    module_name: &str,
) -> Result<module_config::Model, StoreError> {
    ModuleConfig::find()
        .filter(module_config::Column::ModuleName.eq(module_name))
        .one(db)
        .await?
        .ok_or_else(|| StoreError::NotFound(format!("module config for: {}", module_name)))
}

pub async fn list_all(db: &DbConn) -> Result<Vec<module_config::Model>, StoreError> {
    let configs = ModuleConfig::find().all(db).await?;
    Ok(configs)
}

pub async fn update_config_value(
    db: &DbConn,
    id: &Uuid,
    config_value: serde_json::Value,
) -> Result<module_config::Model, StoreError> {
    let existing = ModuleConfig::find_by_id(*id)
        .one(db)
        .await?
        .ok_or_else(|| StoreError::NotFound(format!("module config with id: {}", id)))?;

    let now = Utc::now().fixed_offset();
    let mut active_config: module_config::ActiveModel = existing.into();
    active_config.config_value = Set(config_value);
    active_config.updated_at = Set(now);

    Ok(active_config.update(db).await?)
}

pub async fn upsert(
    db: &DbConn,
    module_name: &str,
    config_value: serde_json::Value,
    json_schema: serde_json::Value,
) -> Result<(), StoreError> {
    let now = Utc::now();
    let existing = ModuleConfig::find()
        .filter(module_config::Column::ModuleName.eq(module_name))
        .one(db)
        .await?;

    if let Some(record) = existing {
        let mut active_model: module_config::ActiveModel = record.into();
        active_model.config_value = Set(config_value);
        active_model.json_schema = Set(json_schema);
        active_model.updated_at = Set(now.into());
        active_model.update(db).await?;
    } else {
        let active_model = module_config::ActiveModel {
            id: Set(Uuid::new_v4()),
            module_name: Set(module_name.to_string()),
            config_value: Set(config_value),
            json_schema: Set(json_schema),
            created_at: Set(now.into()),
            updated_at: Set(now.into()),
        };
        active_model.insert(db).await?;
    }

    Ok(())
}
