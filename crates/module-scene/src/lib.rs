pub mod error;
pub mod func;

use crate::error::*;
use sea_orm::DatabaseConnection;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 场景模块配置
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct SceneConfig {}

/// 场景管理模块
pub struct Scene {
    db: DatabaseConnection,
}

impl Scene {
    pub async fn init_from_file_config(db: DatabaseConnection) -> Result<Self> {
        Self::init(SceneConfig {}, db).await
    }

    pub async fn init_from_db_config(db: DatabaseConnection) -> Result<Self> {
        Self::init(
            nihility_config::get_config_with_db::<SceneConfig>(env!("CARGO_PKG_NAME"), &db)
                .await?,
            db,
        )
        .await
    }

    pub async fn init(_config: SceneConfig, db: DatabaseConnection) -> Result<Self> {
        Ok(Self { db })
    }
}
