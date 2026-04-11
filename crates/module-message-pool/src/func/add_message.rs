use crate::{Message, MessagePool, MessagePoolError};
use chrono::Utc;
use nihility_server_entity::message;
use schemars::JsonSchema;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 添加消息参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddMessagesParam {
    /// 场景 ID
    pub scene_id: String,
    /// 消息列表
    pub messages: Vec<Message>,
}

/// 添加消息返回结果
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddMessagesResult {
    /// 消息 ID 列表
    pub message_ids: Vec<String>,
    /// 创建时间列表
    pub created_at: Vec<String>,
}

impl MessagePool {
    /// 添加消息列表到消息池
    pub async fn add_messages(
        &self,
        param: AddMessagesParam,
    ) -> Result<AddMessagesResult, MessagePoolError> {
        // Validate scene exists
        let scene_uuid = Uuid::parse_str(&param.scene_id)
            .map_err(|_| MessagePoolError::SceneNotFound(param.scene_id.clone()))?;

        // Check scene exists
        let scene = nihility_server_entity::scene::Entity::find_by_id(scene_uuid)
            .one(&self.conn)
            .await?
            .ok_or_else(|| MessagePoolError::SceneNotFound(param.scene_id.clone()))?;

        let mut message_ids = Vec::new();
        let mut created_ats = Vec::new();

        for msg in &param.messages {
            let now = Utc::now();
            let message_id = Uuid::new_v4();

            // Create message content JSON
            let content_json = serde_json::to_value(&msg.content)?;

            // Create metadata JSON
            let metadata_json = serde_json::to_value(&msg.metadata)?;

            // Build active model
            let active_model = message::ActiveModel {
                id: Set(message_id),
                scene_id: Set(scene.id),
                msg_type: Set(msg.content.to_msg_type()),
                content: Set(content_json.into()),
                metadata: Set(metadata_json.into()),
                is_processed: Set(false),
                created_at: Set(now.into()),
                updated_at: Set(now.into()),
            };

            // Insert into database
            active_model.insert(&self.conn).await?;

            message_ids.push(message_id.to_string());
            created_ats.push(now.to_rfc3339());

            // Trigger async analysis chain (发送消息Id到分析通道)
            self.trigger_analysis(param.scene_id.clone(), message_id.to_string());
        }

        Ok(AddMessagesResult {
            message_ids,
            created_at: created_ats,
        })
    }
}