use crate::config::ModelCapability;
use crate::error::Result;
use crate::provider;
use crate::ModelModule;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 语音识别请求参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SpeechRecognitionParam {
    /// PCM 原始音频数据（经过 VAD 处理后的数据）
    pub audio_data: Vec<u8>,
    /// 采样率 (如 16000, 44100)
    pub sample_rate: u32,
    /// 声道数 (1 为单声道, 2 为立体声)
    pub channels: u8,
    /// 位深 (如 16)
    pub bits_per_sample: u8,
}

impl ModelModule {
    /// 语音识别
    pub async fn speech_recognition(&self, param: SpeechRecognitionParam) -> Result<String> {
        self.pool
            .invoke(ModelCapability::SpeechRecognition, |model| {
                let audio_data = param.audio_data.clone();
                async move {
                    let provider = provider::ProviderFactory::create(&model.provider)?;
                    provider
                        .speech_recognition(
                            &audio_data,
                            param.sample_rate,
                            param.channels,
                            param.bits_per_sample,
                        )
                        .await
                }
            })
            .await
    }
}
