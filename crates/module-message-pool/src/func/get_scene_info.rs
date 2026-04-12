use crate::{MessagePool, MessagePoolError, SceneInfo};
use nihility_store_operate::scene;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 获取场景信息参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetSceneInfoParam {
    /// 场景 ID
    pub scene_id: Uuid,
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
        let scene = scene::find_scene_by_id(&self.conn, param.scene_id)
            .await
            .map_err(|_| MessagePoolError::SceneNotFound(param.scene_id))?;

        let children_ids = if param.include_children {
            self.fetch_direct_children_ids(param.scene_id).await?
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
        let children = scene::find_scenes_by_parent_id(&self.conn, parent_id).await?;

        Ok(children.into_iter().map(|c| c.id.to_string()).collect())
    }
}
