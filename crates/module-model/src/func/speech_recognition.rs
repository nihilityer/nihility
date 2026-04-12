use crate::config::ModelCapability;
use crate::Model;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 语音识别请求参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SpeechRecognitionParam {
    /// f32 归一化音频数据（32-bit float PCM，经过 VAD 处理后的数据）
    pub audio_data: Vec<f32>,
}

impl Model {
    /// 语音识别
    pub async fn speech_recognition(
        &self,
        param: SpeechRecognitionParam,
    ) -> crate::error::Result<String> {
        self.pool
            .invoke(ModelCapability::SpeechRecognition, |provider| {
                let audio_data = param.audio_data.clone();
                async move { provider.speech_recognition(&audio_data).await }
            })
            .await
    }
}
