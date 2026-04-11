use crate::{MessagePool, MessagePoolError, SceneInfo};
use nihility_server_entity::prelude::Scene;
use nihility_server_entity::scene;
use schemars::JsonSchema;
use sea_orm::QueryFilter;
use sea_orm::{ColumnTrait, EntityTrait};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 获取场景信息参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetSceneInfoParam {
    /// 场景 ID
    pub scene_id: String,
    /// 是否包含直接子场景 ID 列表（默认 true）
    #[serde(default = "default_include_children")]
    pub include_children: bool,
}

fn default_include_children() -> bool {
    true
}

impl MessagePool {
    /// 获取场景信息
    pub async fn get_scene_info(
        &self,
        param: GetSceneInfoParam,
    ) -> Result<SceneInfo, MessagePoolError> {
        let scene_uuid = Uuid::parse_str(&param.scene_id)
            .map_err(|_| MessagePoolError::SceneNotFound(param.scene_id.clone()))?;

        // Fetch current scene
        let scene = Scene::find_by_id(scene_uuid)
            .one(&self.conn)
            .await?
            .ok_or_else(|| MessagePoolError::SceneNotFound(param.scene_id.clone()))?;

        // Fetch direct children IDs if requested
        let children_ids = if param.include_children {
            self.fetch_direct_children_ids(scene_uuid).await?
        } else {
            vec![]
        };

        Ok(SceneInfo {
            id: scene.id.to_string(),
            name: scene.name.clone(),
            parent_id: scene.parent_id.map(|p| p.to_string()),
            metadata: scene.metadata,
            children_ids,
        })
    }

    /// 获取直接子场景 ID 列表（非递归）
    async fn fetch_direct_children_ids(
        &self,
        parent_id: Uuid,
    ) -> Result<Vec<String>, MessagePoolError> {
        let children = Scene::find()
            .filter(scene::Column::ParentId.eq(parent_id))
            .all(&self.conn)
            .await?;

        Ok(children
            .into_iter()
            .map(|c| c.id.to_string())
            .collect())
    }
}
