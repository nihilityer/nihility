use crate::{Message, MessagePool, MessagePoolError};
use nihility_store_operate::message;
use nihility_store_operate::scene;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 添加消息参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddMessagesParam {
    /// 场景 ID
    pub scene_id: Uuid,
    /// 消息列表
    pub messages: Vec<Message>,
}

/// 添加消息返回结果
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddMessagesResult {
    /// 消息 ID 列表
    pub message_ids: Vec<String>,
}

impl MessagePool {
    /// 添加消息列表到消息池
    pub async fn add_messages(
        &self,
        param: AddMessagesParam,
    ) -> Result<AddMessagesResult, MessagePoolError> {
        let _scene = scene::find_scene_by_id(&self.conn, param.scene_id).await?;

        let mut message_ids = Vec::new();

        let group_id = Uuid::new_v4();
        for msg in &param.messages {
            let content_json = serde_json::to_value(&msg.content)?;

            let metadata_json = serde_json::to_value(&msg.metadata)?;

            let message = message::insert_message(
                &self.conn,
                param.scene_id,
                msg.content.to_msg_type(),
                content_json,
                metadata_json,
                group_id,
                false,
            )
            .await?;

            message_ids.push(message.id.to_string());

            self.trigger_analysis(param.scene_id, message.id);
        }

        Ok(AddMessagesResult { message_ids })
    }
}
