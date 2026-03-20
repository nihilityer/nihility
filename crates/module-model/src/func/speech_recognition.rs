use crate::config::ModelCapability;
use crate::error::ModelError;
use crate::error::Result;
use crate::provider;
use crate::ModelModule;
use base64::Engine;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 语音识别请求参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SpeechRecognitionParam {
    /// Base64 编码的音频数据
    pub audio_data: String,
    /// 音频格式 (mp3, wav, flac 等)
    pub format: Option<String>,
}

impl ModelModule {
    /// 语音识别
    pub async fn speech_recognition(
        &self,
        param: &SpeechRecognitionParam,
    ) -> Result<String> {
        let audio_bytes = base64::engine::general_purpose::STANDARD
            .decode(&param.audio_data)
            .map_err(|e| ModelError::AudioDecode(e.to_string()))?;
        let format = param.format.clone().unwrap_or_else(|| "mp3".to_string());
        self.pool
            .invoke(ModelCapability::SpeechRecognition, move |model| {
                let audio_bytes = audio_bytes.clone();
                let format = format.clone();
                async move {
                    let provider = provider::ProviderFactory::create(&model.provider)?;
                    provider.speech_recognition(&audio_bytes, &format).await
                }
            })
            .await
    }
}
