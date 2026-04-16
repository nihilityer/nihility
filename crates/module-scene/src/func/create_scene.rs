use crate::error::*;
use crate::{SceneManager, SceneMetadata};
use nihility_store_operate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// 创建场景参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CreateSceneParam {
    /// 场景名称（唯一）
    pub name: String,
    /// 父场景ID（可选，用于层级结构）
    pub parent_id: Option<Uuid>,
    /// 场景元数据
    pub metadata: SceneMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSceneResult {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub metadata: Value,
}

impl SceneManager {
    pub async fn create_scene(&self, param: CreateSceneParam) -> Result<CreateSceneResult> {
        let model = nihility_store_operate::scene::insert_scene(
            &self.db,
            param.name,
            param.parent_id,
            serde_json::to_value(&param.metadata)?,
        )
        .await?;

        Ok(CreateSceneResult {
            id: model.id,
            name: model.name,
            parent_id: model.parent_id,
            metadata: model.metadata,
        })
    }
}
