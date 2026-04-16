use crate::error::*;
use crate::Scene;
use nihility_store_operate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 删除场景参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DeleteSceneParam {
    /// 场景ID
    pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSceneResult {
    pub success: bool,
    pub id: Uuid,
}

impl Scene {
    pub async fn delete_scene(&self, param: DeleteSceneParam) -> Result<DeleteSceneResult> {
        nihility_store_operate::scene::delete_scene(&self.db, param.id).await?;
        Ok(DeleteSceneResult {
            success: true,
            id: param.id,
        })
    }
}
