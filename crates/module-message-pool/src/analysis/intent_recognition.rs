use crate::MessagePoolError;
use async_trait::async_trait;
use nihility_store_operate::message;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::analysis::Analyzer;

/// 意图识别器
/// 使用提示词方式进行意图识别
pub struct IntentAnalyzer {
    priority: i32,
}

impl IntentAnalyzer {
    pub fn new(priority: i32) -> Self {
        Self { priority }
    }
}

#[async_trait]
impl Analyzer for IntentAnalyzer {
    fn name(&self) -> &str {
        "intent_recognition"
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

        // TODO: 实现意图识别逻辑
        // 可以遍历消息内容，使用 AI 模型进行意图识别

        tracing::info!(
            "IntentAnalyzer processed {} messages in group_id {}",
            messages.len(),
            group_id
        );

        // 意图识别不终止分析链
        Ok(true)
    }
}
