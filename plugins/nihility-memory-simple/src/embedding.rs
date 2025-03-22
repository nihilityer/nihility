use async_openai::{
    types::{CreateEmbeddingRequestArgs},
    Client,
};
use async_openai::config::OpenAIConfig;
use anyhow::Result;
use tracing::debug;

#[derive(Clone)]
pub struct EmbeddingClient {
    client: Client<OpenAIConfig>,
    model: String,
}

impl EmbeddingClient {
    pub fn new(config: OpenAIConfig, model: String) -> Self {
        let client = Client::with_config(config);
        Self { client, model }
    }

    pub async fn get_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let request = CreateEmbeddingRequestArgs::default()
            .model(&self.model)
            .input(text)
            .build()?;
        debug!("request: {:?}", request);
        let response = self.client.embeddings().create(request).await?;
        // debug!("response: {:?}", response);
        Ok(response.data[0].embedding.clone())
    }
}