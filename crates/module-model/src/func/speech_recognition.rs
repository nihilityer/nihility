use crate::config::ModelCapability;
use crate::ModelModule;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 语音识别请求参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SpeechRecognitionParam {
    /// f32 归一化音频数据（32-bit float PCM，经过 VAD 处理后的数据）
    pub audio_data: Vec<f32>,
    /// 采样率 (如 16000, 44100)
    pub sample_rate: u32,
    /// 声道数 (1 为单声道, 2 为立体声)
    pub channels: u8,
}

impl ModelModule {
    /// 语音识别
    pub async fn speech_recognition(
        &self,
        param: SpeechRecognitionParam,
    ) -> crate::error::Result<String> {
        let audio_module = self.audio_module.clone().ok_or_else(|| {
            crate::error::ModelError::Provider("audio_module not set".to_string())
        })?;
        self.pool
            .invoke(ModelCapability::SpeechRecognition, |provider| {
                let audio_data = param.audio_data.clone();
                let audio_module = audio_module.clone();
                async move {
                    provider
                        .speech_recognition(
                            &audio_data,
                            param.sample_rate,
                            param.channels,
                            &audio_module,
                        )
                        .await
                }
            })
            .await
    }
}
