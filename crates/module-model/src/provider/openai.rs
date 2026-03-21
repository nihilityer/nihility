use crate::config::OpenAIConfig as ProviderConfig;
use crate::error::{ModelError, Result};
use crate::provider::ModelProvider;
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::{
    ChatCompletionRequestMessageContentPartImage, ChatCompletionRequestMessageContentPartText,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs, ImageUrl, Prompt,
};
use async_openai::types::completions::CreateCompletionRequestArgs;
use async_openai::Client;
use async_trait::async_trait;
use std::sync::Arc;

/// OpenAI API Provider 实现
pub struct OpenAiProvider {
    client: Client<Arc<dyn async_openai::config::Config>>,
    model: String,
}

impl OpenAiProvider {
    pub fn new(config: &ProviderConfig) -> Result<Self> {
        let openai_config = OpenAIConfig::new()
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
            .map_err(|e| ModelError::ApiRequest(e.to_string()))?;

        let response = self
            .client
            .completions()
            .create(request)
            .await
            .map_err(|e| ModelError::ApiRequest(e.to_string()))?;

        let content = response
            .choices
            .first()
            .map(|c| c.text.as_str())
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    async fn image_understanding(&self, image_url: &str, prompt: &str) -> Result<String> {
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages([ChatCompletionRequestUserMessageArgs::default()
                .content(vec![
                    ChatCompletionRequestMessageContentPartText::from(prompt).into(),
                    ChatCompletionRequestMessageContentPartImage::from(ImageUrl {
                        url: image_url.to_string(),
                        detail: None,
                    })
                    .into(),
                ])
                .build()
                .map_err(|e| ModelError::ApiRequest(e.to_string()))?
                .into()])
            .stream(false)
            .build()
            .map_err(|e| ModelError::ApiRequest(e.to_string()))?;

        let response = self
            .client
            .chat()
            .create(request)
            .await
            .map_err(|e| ModelError::ApiRequest(e.to_string()))?;

        let content = response
            .choices
            .first()
            .map(|c| c.message.clone())
            .map(|m| m.content)
            .unwrap_or(Some(String::default()))
            .unwrap_or(String::default());

        Ok(content)
    }
}
