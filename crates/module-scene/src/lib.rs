pub mod error;
pub mod func;

use crate::error::*;
use schemars::JsonSchema;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 场景管理模块配置
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct SceneManagerConfig {}

/// 场景管理模块
pub struct SceneManager {
    db: DatabaseConnection,
}

/// 场景
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct Scene {
    pub id: Uuid,
    pub parent_id: Option<Uuid>,
    pub name: String,
    pub metadata: SceneMetadata,
}

/// 场景元数据
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
pub struct SceneMetadata {
    pub description: String,
}

impl SceneManager {
    pub async fn init_from_file_config(db: DatabaseConnection) -> Result<Self> {
        Self::init(SceneManagerConfig {}, db).await
    }

    pub async fn init_from_db_config(db: DatabaseConnection) -> Result<Self> {
        Self::init(
            nihility_config::get_config_with_db::<SceneManagerConfig>(env!("CARGO_PKG_NAME"), &db)
                .await?,
            db,
        )
        .await
    }

    pub async fn init(_config: SceneManagerConfig, db: DatabaseConnection) -> Result<Self> {
        Ok(Self { db })
    }
}
