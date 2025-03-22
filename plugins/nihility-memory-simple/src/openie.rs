use anyhow::Result;
use async_openai::config::OpenAIConfig;
use async_openai::types::{ChatCompletionRequestSystemMessage, ResponseFormat};
use async_openai::{Client, types::ChatCompletionRequestMessage};
use serde::de::Error;
use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgeTriple {
    pub subject: String,
    pub relation: String,
    pub object: String,
    pub confidence: f32,
}

#[derive(Clone)]
pub struct OpenIEProcessor {
    client: Client<OpenAIConfig>,
    model: String,
}

impl OpenIEProcessor {
    pub fn new(config: OpenAIConfig, model: String) -> Self {
        let client = Client::with_config(config);
        Self { client, model }
    }

    pub async fn extract_triples<T: Into<String>>(&self, text: T) -> Result<Vec<KnowledgeTriple>> {
        let prompt = format!(
            r#"
        Analyze the following text and extract all factual relationships in JSON format.
        Use strict schema: [{{"subject": "...", "relation": "...", "object": "...", "confidence": 0.0}}]
        Text: {}
        "#,
            text.into()
        );

        let messages = vec![
            ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: "You are a precise information extraction system. Extract relationships with confidence scores.".into(),
                name: None,
            }),
            ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: prompt.into(),
                name: None,
            })
        ];

        let response = self
            .client
            .chat()
            .create(
                async_openai::types::CreateChatCompletionRequestArgs::default()
                    .model(&self.model)
                    .messages(messages)
                    .response_format(ResponseFormat::JsonObject)
                    .temperature(0.0)
                    .build()?,
            )
            .await?;
        debug!("response: {:?}", response);

        let json_str = response.choices[0]
            .message
            .content
            .as_ref()
            .ok_or(serde_json::Error::custom("No content"))?;
        debug!("json_str: {}", json_str);
        let triples: Vec<KnowledgeTriple> = serde_json::from_str(json_str.as_str())?;
        Ok(triples)
    }
}
