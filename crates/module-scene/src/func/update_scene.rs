use crate::error::*;
use crate::Scene;
use nihility_store_operate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;
use uuid::Uuid;

/// 更新场景参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UpdateSceneParam {
    /// 场景ID
    pub id: Uuid,
    /// 场景名称（可选）
    pub name: Option<String>,
    /// 父场景ID（可选）
    pub parent_id: Option<Uuid>,
    /// 场景Json元数据（可选）
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSceneResult {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

impl Scene {
    pub async fn update_scene(&self, param: UpdateSceneParam) -> Result<UpdateSceneResult> {
        let metadata = if let Some(metadata) = &param.metadata {
            Some(Value::from_str(metadata)?)
        } else {
            None
        };
        let model = nihility_store_operate::scene::update_scene(
            &self.db,
            param.id,
            param.name,
            param.parent_id,
            metadata,
        )
        .await?;

        let metadata: Value = model.metadata;

        Ok(UpdateSceneResult {
            id: model.id,
            name: model.name,
            parent_id: model.parent_id,
            metadata,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        })
    }
}
