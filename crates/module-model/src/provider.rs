use crate::config::ProviderType;
use crate::error::Result;
use async_trait::async_trait;

mod openai;

/// 模型 provider trait
#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// 文本补全
    async fn text_completion(&self, prompt: &str) -> Result<String>;

    /// 图片理解
    async fn image_understanding(&self, image_url: &str, prompt: &str) -> Result<String>;

    /// 语音识别
    async fn speech_recognition(&self, audio_data: &[u8], format: &str) -> Result<String>;
}

/// Provider 工厂
pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create(provider_type: &ProviderType) -> Result<Box<dyn ModelProvider>> {
        match provider_type {
            ProviderType::OpenAI(config) => Ok(Box::new(openai::OpenAiProvider::new(config)?)),
            ProviderType::Embed(_) => Err(crate::error::ModelError::Provider(
                "Embed provider not implemented yet".to_string(),
            )),
        }
    }
}
