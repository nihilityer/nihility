use crate::error::*;
use chrono::Utc;
use nihility_server_entity::module_config;
use nihility_server_entity::prelude::ModuleConfig;
use sea_orm::{ActiveModelTrait, ColumnTrait, DbConn, EntityTrait, QueryFilter, Set};
use serde_json::Value;
use uuid::Uuid;

pub struct ModuleConfigService;

impl ModuleConfigService {
    /// 获取所有模块配置
    pub async fn list_all(db: &DbConn) -> Result<Vec<module_config::Model>> {
        let configs = ModuleConfig::find().all(db).await?;
        Ok(configs)
    }

    /// 根据模块名称查找配置
    pub async fn find_by_module_name(db: &DbConn, module_name: &str) -> Result<module_config::Model> {
        match ModuleConfig::find()
            .filter(module_config::Column::ModuleName.eq(module_name))
            .one(db)
            .await?
        {
            None => Err(NihilityServerError::NotFound(format!(
                "module config for: {}",
                module_name
            ))),
            Some(record) => Ok(record),
        }
    }

    /// 根据 ID 查找配置
    pub async fn find_by_id(db: &DbConn, id: &Uuid) -> Result<module_config::Model> {
        match ModuleConfig::find_by_id(*id).one(db).await? {
            None => Err(NihilityServerError::NotFound(format!(
                "module config with id: {}",
                id
            ))),
            Some(record) => Ok(record),
        }
    }

    /// 更新模块配置
    pub async fn update(
        db: &DbConn,
        id: &Uuid,
        config_value: Value,
    ) -> Result<module_config::Model> {
        let existing = Self::find_by_id(db, id).await?;

        let now = Utc::now().fixed_offset();
        let mut active_config: module_config::ActiveModel = existing.into();
        active_config.config_value = Set(config_value);
        active_config.updated_at = Set(now);

        let updated_config = active_config.update(db).await?;
        Ok(updated_config)
    }
}
