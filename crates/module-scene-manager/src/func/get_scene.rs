use crate::error::*;
use crate::{SceneManager, SceneMetadata};
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
    pub metadata: SceneMetadata,
}

impl SceneManager {
    pub async fn get_scene(&self, param: GetSceneParam) -> Result<SceneResponse> {
        let model = nihility_store_operate::scene::find_scene_by_id(&self.db, param.id).await?;

        Ok(SceneResponse {
            id: model.id,
            name: model.name,
            parent_id: model.parent_id,
            metadata: serde_json::from_value(model.metadata)?,
        })
    }
}
