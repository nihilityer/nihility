use crate::{ContentData, MessagePoolError};
use async_trait::async_trait;
use nihility_store_operate::message;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::analysis::Analyzer;

/// 命令分析器
/// 检查消息是否以 `/` 开头，如果是则终止分析链
pub struct CommandAnalyzer {
    priority: i32,
}

impl CommandAnalyzer {
    pub fn new(priority: i32) -> Self {
        Self { priority }
    }
}

#[async_trait]
impl Analyzer for CommandAnalyzer {
    fn name(&self) -> &str {
        "command_analysis"
    }

    fn priority(&self) -> i32 {
        self.priority
    }

    async fn analyze(
        &self,
        db: &DatabaseConnection,
        group_id: Uuid,
    ) -> Result<bool, MessagePoolError> {
        // 根据 group_id 拉取所有消息
        let messages = message::find_message_by_group_id(db, group_id).await?;

        for msg in messages {
            let content: ContentData = serde_json::from_value(msg.content)?;

            // 检查是否是文本消息且以 `/` 开头
            if let ContentData::Text { body } = content
                && body.starts_with('/')
            {
                tracing::info!(
                    "Command detected in group_id {}: {}",
                    group_id,
                    body.split_whitespace().next().unwrap_or(&body)
                );
                // 命令分析器检测到命令，终止后续分析链
                return Ok(false);
            }
        }

        // 未检测到命令，继续分析链
        Ok(true)
    }
}
