use crate::{MessagePool, MessagePoolError};
use nihility_store_operate::message;
use nihility_store_operate::scene;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 处理场景消息参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProcessSceneMessagesParam {
    /// 场景 ID
    pub scene_id: Uuid,
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
    pub processed_message_ids: Vec<Uuid>,
}

impl MessagePool {
    /// 处理场景中所有未处理的消息
    pub async fn process_scene_messages(
        &self,
        param: ProcessSceneMessagesParam,
    ) -> Result<ProcessMessagesResult, MessagePoolError> {
        // Collect scene IDs to process (self + direct children if requested)
        let mut scene_ids = vec![param.scene_id];
        if param.process_children {
            scene_ids.extend(self.collect_direct_child_scene_ids(param.scene_id).await?);
        }

        // Find all unprocessed messages in these scenes using store_operate
        let messages =
            message::find_unprocessed_messages_by_scene_ids(&self.conn, &scene_ids).await?;

        let mut processed_ids = Vec::new();

        // Mark each message as processed using store_operate
        for msg in messages {
            processed_ids.push(msg.id);
            message::update_message_processed(&self.conn, msg.id, true).await?;
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
        let children = scene::find_scenes_by_parent_id(&self.conn, parent_id).await?;

        Ok(children.into_iter().map(|c| c.id).collect())
    }
}
