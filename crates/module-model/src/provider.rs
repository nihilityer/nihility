use crate::config::ProviderType;
use crate::error::{ModelError, Result};
use async_trait::async_trait;
use futures::Stream;
use std::pin::Pin;
use std::sync::Arc;
use tracing::debug;

pub(crate) mod openai_api;
pub(crate) mod sense_voice;

pub type BoxStream<T> = Pin<Box<dyn Stream<Item = Result<T>> + Send + 'static>>;

/// 模型 provider trait
#[async_trait]
pub trait ModelProvider: Send + Sync {
    /// 文本补全
    async fn text_completion(&self, prompt: &str) -> Result<String> {
        debug!("text_completion: {}", prompt);
        Err(ModelError::Unsupported(
            "text_completion is not supported".to_string(),
        ))
    }

    /// 文本补全流式响应
    async fn text_completion_stream(&self, prompt: &str) -> Result<BoxStream<String>> {
        debug!("text_completion_stream: {}", prompt);
        Err(ModelError::Unsupported(
            "text_completion_stream is not supported".to_string(),
        ))
    }

    /// 图片理解
    async fn image_understanding(&self, image_url: &str, prompt: &str) -> Result<String> {
        debug!(
            "image_understanding: image_url: {}, prompt: {}",
            image_url, prompt
        );
        Err(ModelError::Unsupported(
            "image_understanding is not supported".to_string(),
        ))
    }

    /// 图片理解流式响应
    async fn image_understanding_stream(
        &self,
        image_url: &str,
        prompt: &str,
    ) -> Result<BoxStream<String>> {
        debug!(
            "image_understanding_stream: image_url: {}, prompt: {}",
            image_url, prompt
        );
        Err(ModelError::Unsupported(
            "image_understanding_stream is not supported".to_string(),
        ))
    }

    /// 语音识别
    async fn speech_recognition(
        &self,
        audio_data: &[f32],
        sample_rate: u32,
        channels: u8,
        _audio_module: &Arc<nihility_module_audio::AudioModule>,
    ) -> Result<String> {
        debug!(
            "speech_recognition: sample_rate: {}, channels: {}, data_len: {}",
            sample_rate,
            channels,
            audio_data.len()
        );
        Err(ModelError::Unsupported(
            "speech_recognition is not supported".to_string(),
        ))
    }
}

/// Provider 工厂
pub struct ProviderFactory;

impl ProviderFactory {
    pub fn create(provider_type: &ProviderType) -> Result<Box<dyn ModelProvider>> {
        match provider_type {
            ProviderType::OpenAI(config) => {
                Ok(Box::new(openai_api::OpenAiApiProvider::new(config)?))
            }
            ProviderType::Embed(embed) => match embed {
                crate::config::EmbedProvider::SenseVoice(cfg) => {
                    Ok(Box::new(sense_voice::SenseVoice::init(cfg.clone())?))
                }
            },
        }
    }
}
