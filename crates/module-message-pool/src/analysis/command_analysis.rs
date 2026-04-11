use crate::{ContentData, MessageMetadata, MessagePoolError};
use async_trait::async_trait;

use crate::analysis::{AnalysisContext, Analyzer};

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
        _content: &ContentData,
        _metadata: &MessageMetadata,
        _context: &AnalysisContext,
    ) -> Result<bool, MessagePoolError> {
        todo!()
    }
}
