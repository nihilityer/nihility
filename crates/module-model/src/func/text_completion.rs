use crate::config::ModelCapability;
use crate::error::Result;
use crate::provider;
use crate::ModelModule;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 文本补全请求参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TextCompletionParam {
    /// 提示词
    pub prompt: String,
}

impl ModelModule {
    /// 文本补全
    pub async fn text_completion(&self, param: &TextCompletionParam) -> Result<String> {
        self.pool
            .invoke(ModelCapability::TextCompletion, move |model| async move {
                let provider = provider::ProviderFactory::create(&model.provider)?;
                provider.text_completion(&param.prompt).await
            })
            .await
    }
}
