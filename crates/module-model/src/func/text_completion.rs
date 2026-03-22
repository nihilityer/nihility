use crate::config::ModelCapability;
use crate::error::Result;
use crate::provider::BoxStream;
use crate::ModelModule;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 文本补全请求参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TextCompletionParam {
    /// 提示词
    pub prompt: String,
}

/// 文本补全流式请求参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TextCompletionStreamParam {
    /// 提示词
    pub prompt: String,
}

impl ModelModule {
    /// 文本补全
    pub async fn text_completion(&self, param: &TextCompletionParam) -> Result<String> {
        self.pool
            .invoke(ModelCapability::TextCompletion, |provider| async move {
                provider.text_completion(&param.prompt).await
            })
            .await
    }

    /// 文本补全流式响应
    pub async fn text_completion_stream(
        &self,
        param: &TextCompletionStreamParam,
    ) -> Result<BoxStream<String>> {
        self.pool
            .invoke(ModelCapability::TextCompletion, |provider| async move {
                provider.text_completion_stream(&param.prompt).await
            })
            .await
    }
}
