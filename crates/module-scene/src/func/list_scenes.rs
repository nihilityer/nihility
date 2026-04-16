use crate::error::*;
use crate::Scene;
use nihility_store_operate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 列出场景参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListScenesParam {
    /// 父场景ID（可选，为空则列出根场景）
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneItem {
    pub id: Uuid,
    pub name: String,
    pub parent_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSccenesResult {
    pub scenes: Vec<SceneItem>,
    pub total: usize,
}

impl Scene {
    pub async fn list_scenes(&self, param: ListScenesParam) -> Result<ListSccenesResult> {
        let scenes = if let Some(parent_id) = param.parent_id {
            nihility_store_operate::scene::find_scenes_by_parent_id(&self.db, parent_id).await?
        } else {
            nihility_store_operate::scene::find_all_scenes(&self.db).await?
        };

        let items: Vec<SceneItem> = scenes
            .into_iter()
            .map(|s| {
                let metadata: serde_json::Value = s.metadata;
                SceneItem {
                    id: s.id,
                    name: s.name,
                    parent_id: s.parent_id,
                    metadata,
                    created_at: s.created_at.to_rfc3339(),
                    updated_at: s.updated_at.to_rfc3339(),
                }
            })
            .collect();

        let total = items.len();
        Ok(ListSccenesResult {
            scenes: items,
            total,
        })
    }
}
