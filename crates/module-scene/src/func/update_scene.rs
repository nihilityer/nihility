use crate::error::*;
use crate::{SceneManager, SceneMetadata};
use nihility_store_operate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
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
    /// 场景元数据（可选）
    pub metadata: Option<SceneMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSceneResult {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub metadata: SceneMetadata,
}

impl SceneManager {
    pub async fn update_scene(&self, param: UpdateSceneParam) -> Result<UpdateSceneResult> {
        let metadata = if let Some(metadata) = &param.metadata {
            Some(serde_json::to_value(metadata)?)
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

        Ok(UpdateSceneResult {
            id: model.id,
            name: model.name,
            parent_id: model.parent_id,
            metadata: serde_json::from_value(model.metadata)?,
        })
    }
}
