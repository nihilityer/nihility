use anyhow::Result;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::ChatCompletionRequestUserMessageContent::Array;
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartImage,
    ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContentPart, CreateChatCompletionRequestArgs,
    CreateEmbeddingRequestArgs, ResponseFormat,
};
use async_trait::async_trait;
use nihility_common::config::get_config;
use nihility_common::model::NihilityModel;
use nihility_common::set_model;
use serde::de::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NihilityApiModelConfig {
    pub embedding_model: String,
    pub chat_completion_model: String,
    pub vl_model: String,
    pub api_base_url: String,
    pub api_key: String,
}

impl Default for NihilityApiModelConfig {
    fn default() -> Self {
        Self {
            embedding_model: "text-embedding-v3".to_string(),
            chat_completion_model: "qwen-plus".to_string(),
            vl_model: "text-vl".to_string(),
            api_base_url: "https://dashscope.aliyuncs.com/compatible-mode/v1".to_string(),
            api_key: "".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct NihilityApiModel {
    config: NihilityApiModelConfig,
    client: Client<OpenAIConfig>,
}

impl NihilityApiModel {
    pub async fn init() -> Result<()> {
        let config =
            get_config::<NihilityApiModelConfig>(env!("CARGO_PKG_NAME").to_string()).await?;
        let openai_config = OpenAIConfig::new()
            .with_api_base(&config.api_base_url)
            .with_api_key(&config.api_key);
        let client = Client::with_config(openai_config);
        let model = Self {
            config: config.clone(),
            client,
        };
        set_model(Box::new(model)).await;
        Ok(())
    }
}

#[async_trait]
impl NihilityModel for NihilityApiModel {
    async fn get_embedding(&self, text: &String) -> Result<Vec<f32>> {
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.config.embedding_model)
            .input(text)
            .build()?;
        debug!("request: {:?}", request);
        let response = self.client.embeddings().create(request).await?;
        debug!("response: {:?}", response);
        Ok(response.data[0].embedding.clone())
    }

    async fn get_chat_completion(&self, system: String, user: String) -> Result<Value> {
        let messages = vec![
            ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: system.into(),
                name: None,
            }),
            ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                content: user.into(),
                name: None,
            }),
        ];
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.config.chat_completion_model)
            .messages(messages)
            .response_format(ResponseFormat::JsonObject)
            .temperature(0.0)
            .build()?;
        debug!("request: {:?}", request);
        let response = self.client.chat().create(request).await?;
        debug!("response: {:?}", response);
        let json_str = response.choices[0]
            .message
            .content
            .as_ref()
            .ok_or(serde_json::Error::custom("No content"))?;
        Ok(serde_json::from_str::<Value>(json_str)?)
    }

    async fn get_image_text(&self, system: String, user: String, image: String) -> Result<Value> {
        let messages = vec![
            ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: system.into(),
                name: None,
            }),
            ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                content: Array(vec![
                    ChatCompletionRequestUserMessageContentPart::Text(user.into()),
                    ChatCompletionRequestUserMessageContentPart::ImageUrl(
                        ChatCompletionRequestMessageContentPartImage {
                            image_url: image.into(),
                        },
                    ),
                ]),
                name: None,
            }),
        ];
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.config.vl_model)
            .messages(messages)
            .response_format(ResponseFormat::JsonObject)
            .temperature(0.0)
            .build()?;
        debug!("request: {:?}", request);
        let response = self.client.chat().create(request).await?;
        debug!("response: {:?}", response);
        let json_str = response.choices[0]
            .message
            .content
            .as_ref()
            .ok_or(serde_json::Error::custom("No content"))?;
        Ok(serde_json::from_str::<Value>(json_str)?)
    }
}
