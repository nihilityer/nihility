use crate::error::*;
use crate::Scene;
use nihility_store_operate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 创建场景参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateSceneParam {
    /// 场景名称（唯一）
    pub name: String,
    /// 父场景ID（可选，用于层级结构）
    pub parent_id: Option<Uuid>,
    /// 场景元数据（JSON对象，包含场景描述等）
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSceneResult {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

impl Scene {
    pub async fn create_scene(&self, param: CreateSceneParam) -> Result<CreateSceneResult> {
        let model = nihility_store_operate::scene::insert_scene(
            &self.db,
            param.name,
            param.parent_id,
            param.metadata.clone(),
        )
        .await?;

        Ok(CreateSceneResult {
            id: model.id,
            name: model.name,
            parent_id: model.parent_id,
            metadata: param.metadata,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        })
    }
}
