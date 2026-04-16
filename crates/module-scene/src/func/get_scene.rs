use crate::error::*;
use crate::Scene;
use nihility_store_operate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 获取场景参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetSceneParam {
    /// 场景ID
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneResponse {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

impl Scene {
    pub async fn get_scene(&self, param: GetSceneParam) -> Result<SceneResponse> {
        let model = nihility_store_operate::scene::find_scene_by_id(&self.db, param.id).await?;

        let metadata: serde_json::Value = model.metadata;

        Ok(SceneResponse {
            id: model.id,
            name: model.name,
            parent_id: model.parent_id,
            metadata,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        })
    }
}
