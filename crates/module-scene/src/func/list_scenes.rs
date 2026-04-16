use crate::error::*;
use crate::{SceneManager, SceneMetadata};
use nihility_store_operate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::error;
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
    pub metadata: SceneMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListScenesResult {
    pub scenes: Vec<SceneItem>,
    pub total: usize,
}

impl SceneManager {
    pub async fn list_scenes(&self, param: ListScenesParam) -> Result<ListScenesResult> {
        let scenes = if let Some(parent_id) = param.parent_id {
            nihility_store_operate::scene::find_scenes_by_parent_id(&self.db, parent_id).await?
        } else {
            nihility_store_operate::scene::find_all_scenes(&self.db).await?
        };

        let items: Vec<SceneItem> = scenes
            .into_iter()
            .filter_map(|s| {
                let metadata = match serde_json::from_value::<SceneMetadata>(s.metadata.clone()) {
                    Ok(m) => m,
                    Err(e) => {
                        error!("invalid scene metadata: {}", e);
                        return None;
                    }
                };
                Some(SceneItem {
                    id: s.id,
                    name: s.name,
                    parent_id: s.parent_id,
                    metadata,
                })
            })
            .collect();

        let total = items.len();
        Ok(ListScenesResult {
            scenes: items,
            total,
        })
    }
}
