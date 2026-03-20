use crate::config::OpenAIConfig;
use crate::error::Result;
use crate::provider::ModelProvider;
use async_openai::config::OpenAIConfig as AsyncOpenAIConfig;
use async_openai::types::chat::Prompt;
use async_openai::types::completions::CreateCompletionRequestArgs;
use async_openai::Client;
use async_trait::async_trait;
use std::sync::Arc;

/// OpenAI Provider 实现
pub struct OpenAiProvider {
    client: Client<Arc<dyn async_openai::config::Config>>,
    model: String,
}

impl OpenAiProvider {
    pub fn new(config: &OpenAIConfig) -> Result<Self> {
        let openai_config = AsyncOpenAIConfig::new()
            .with_api_key(&config.api_key)
            .with_api_base(&config.base_url);

        let client =
            Client::with_config(Arc::new(openai_config) as Arc<dyn async_openai::config::Config>);

        Ok(Self {
            client,
            model: config.model.clone(),
        })
    }
}

#[async_trait]
impl ModelProvider for OpenAiProvider {
    async fn text_completion(&self, prompt: &str) -> Result<String> {
        let request = CreateCompletionRequestArgs::default()
            .model(&self.model)
            .prompt(Prompt::String(prompt.to_string()))
            .stream(false)
            .build()
            .map_err(|e| crate::error::ModelError::ApiRequest(e.to_string()))?;

        let response = self
            .client
            .completions()
            .create(request)
            .await
            .map_err(|e| crate::error::ModelError::ApiRequest(e.to_string()))?;

        let content = response
            .choices
            .first()
            .map(|c| c.text.as_str())
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    async fn image_understanding(
        &self,
        _image_url: &str,
        _prompt: &str,
    ) -> Result<String> {
        // 暂时不支持图片理解，返回错误
        Err(crate::error::ModelError::Unsupported(
            "image_understanding is not yet implemented".to_string(),
        ))
    }

    async fn speech_recognition(
        &self,
        _audio_data: &[u8],
        _format: &str,
    ) -> Result<String> {
        // 暂时不支持语音识别，返回错误
        Err(crate::error::ModelError::Unsupported(
            "speech_recognition is not yet implemented".to_string(),
        ))
    }
}
