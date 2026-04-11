use crate::{MessagePool, MessagePoolError};
use nihility_server_entity::message;
use nihility_server_entity::prelude::Message;
use nihility_server_entity::scene::Column as SceneColumn;
use schemars::JsonSchema;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 处理场景消息参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProcessSceneMessagesParam {
    /// 场景 ID
    pub scene_id: String,
    /// 是否同时处理直接子场景（默认 false）
    #[serde(default = "default_process_children")]
    pub process_children: bool,
}

fn default_process_children() -> bool {
    false
}

/// 处理结果
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProcessMessagesResult {
    /// 处理的消息数量
    pub processed_count: usize,
    /// 已处理的消息 ID 列表
    pub processed_message_ids: Vec<String>,
}

impl MessagePool {
    /// 处理场景中所有未处理的消息
    pub async fn process_scene_messages(
        &self,
        param: ProcessSceneMessagesParam,
    ) -> Result<ProcessMessagesResult, MessagePoolError> {
        let scene_uuid = Uuid::parse_str(&param.scene_id)
            .map_err(|_| MessagePoolError::SceneNotFound(param.scene_id.clone()))?;

        // Collect scene IDs to process (self + direct children if requested)
        let mut scene_ids = vec![scene_uuid];
        if param.process_children {
            scene_ids.extend(self.collect_direct_child_scene_ids(scene_uuid).await?);
        }

        // Find all unprocessed messages in these scenes
        let messages = Message::find()
            .filter(message::Column::SceneId.is_in(scene_ids.clone()))
            .filter(message::Column::IsProcessed.eq(false))
            .all(&self.conn)
            .await?;

        let mut processed_ids = Vec::new();

        // Mark each message as processed
        for message in messages {
            processed_ids.push(message.id.to_string());
            let mut active_model = message.into_active_model();
            active_model.is_processed = Set(true);
            active_model.update(&self.conn).await?;
        }

        Ok(ProcessMessagesResult {
            processed_count: processed_ids.len(),
            processed_message_ids: processed_ids,
        })
    }

    /// 收集直接子场景 ID（非递归）
    async fn collect_direct_child_scene_ids(
        &self,
        parent_id: Uuid,
    ) -> Result<Vec<Uuid>, MessagePoolError> {
        let children = nihility_server_entity::scene::Entity::find()
            .filter(SceneColumn::ParentId.eq(parent_id))
            .all(&self.conn)
            .await?;

        Ok(children.into_iter().map(|c| c.id).collect())
    }
}
