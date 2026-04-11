use crate::{ContentData, MessageMetadata, MessagePoolError};
use async_trait::async_trait;

use crate::analysis::{AnalysisContext, Analyzer};

/// 意图识别器
/// 使用提示词方式进行意图识别（提示词模板留空待填充）
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
        _content: &ContentData,
        _metadata: &MessageMetadata,
        _context: &AnalysisContext,
    ) -> Result<bool, MessagePoolError> {
        todo!()
    }
}
