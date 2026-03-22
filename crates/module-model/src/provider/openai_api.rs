use crate::error::{ModelError, Result};
use crate::provider::{BoxStream, ModelProvider};
use async_openai::config::OpenAIConfig;
use async_openai::types::audio::{AudioInput, CreateTranscriptionRequestArgs};
use async_openai::types::chat::{
    ChatCompletionRequestMessageContentPartImage, ChatCompletionRequestMessageContentPartText,
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs, ImageUrl, Prompt,
};
use async_openai::types::completions::CreateCompletionRequestArgs;
use async_openai::types::InputSource;
use async_openai::Client;
use async_trait::async_trait;
use futures::StreamExt;
use hound::{WavSpec, WavWriter};
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiApiConfig {
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

/// OpenAI API Provider 实现
pub struct OpenAiApiProvider {
    client: Client<Arc<dyn async_openai::config::Config>>,
    model: String,
}

impl OpenAiApiProvider {
    pub fn new(config: &OpenAiApiConfig) -> Result<Self> {
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
impl ModelProvider for OpenAiApiProvider {
    async fn text_completion(&self, prompt: &str) -> Result<String> {
        let request = CreateCompletionRequestArgs::default()
            .model(&self.model)
            .prompt(Prompt::String(prompt.to_string()))
            .stream(false)
            .build()?;

        let response = self.client.completions().create(request).await?;

        let content = response
            .choices
            .first()
            .map(|c| c.text.as_str())
            .unwrap_or("")
            .to_string();

        Ok(content)
    }

    async fn text_completion_stream(&self, prompt: &str) -> Result<BoxStream<String>> {
        let request = CreateCompletionRequestArgs::default()
            .model(&self.model)
            .prompt(Prompt::String(prompt.to_string()))
            .stream(true)
            .build()?;

        let mut stream = self.client.completions().create_stream(request).await?;

        let (tx, rx) = mpsc::channel::<Result<String>>(32);

        tokio::spawn(async move {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(response) => {
                        for choice in response.choices {
                            let text = choice.text;
                            let _ = tx.send(Ok(text)).await;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(ModelError::ApiRequest(e))).await;
                        break;
                    }
                }
            }
        });

        let boxed: BoxStream<String> = Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx));
        Ok(boxed)
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
                .build()?
                .into()])
            .stream(false)
            .build()?;

        let response = self.client.chat().create(request).await?;

        let content = response
            .choices
            .first()
            .map(|c| c.message.clone())
            .map(|m| m.content)
            .unwrap_or(Some(String::default()))
            .unwrap_or(String::default());

        Ok(content)
    }

    async fn image_understanding_stream(
        &self,
        image_url: &str,
        prompt: &str,
    ) -> Result<BoxStream<String>> {
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
                .build()?
                .into()])
            .stream(true)
            .build()?;

        let mut stream = self.client.chat().create_stream(request).await?;

        let (tx, rx) = mpsc::channel::<Result<String>>(32);

        tokio::spawn(async move {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(response) => {
                        for choice in response.choices {
                            let delta = choice.delta;
                            if let Some(content) = delta.content {
                                let _ = tx.send(Ok(content)).await;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(ModelError::ApiRequest(e))).await;
                        break;
                    }
                }
            }
        });

        let boxed: BoxStream<String> = Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx));
        Ok(boxed)
    }

    async fn speech_recognition(
        &self,
        audio_data: &[f32],
        sample_rate: u32,
        channels: u8,
        _audio_module: &Arc<nihility_module_audio::AudioModule>,
    ) -> Result<String> {
        // OpenAI expects 16-bit PCM WAV, so convert f32 to 16-bit PCM and create WAV
        let spec = WavSpec {
            channels: channels as u16,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut wav_data = Vec::new();
        {
            let mut writer = WavWriter::new(Cursor::new(&mut wav_data), spec)?;
            for &sample in audio_data {
                let clamped = sample.clamp(-1.0, 1.0);
                let sample_i16 = (clamped * 32767.0).round() as i16;
                writer.write_sample(sample_i16)?;
            }
            writer.finalize()?;
        }

        let request = CreateTranscriptionRequestArgs::default()
            .model(&self.model)
            .file(AudioInput {
                source: InputSource::VecU8 {
                    filename: "audio.wav".to_string(),
                    vec: wav_data,
                },
            })
            .build()?;

        let response = self.client.audio().transcription().create(request).await?;

        Ok(response.text)
    }
}
