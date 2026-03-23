pub mod error;
pub mod func;
pub mod vad;
pub mod vad_stream;

use error::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::vad::silero::Silero;
pub use func::merge_channels::MergeChannelsParam;
pub use func::pcm_to_wav::PcmToWavParam;
pub use vad::silero::SileroConfig;
pub use vad_stream::{SpeechSegment, VadStreamHandler, VadStreamParam};

/// 音频模块配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AudioConfig {
    /// 默认采样率
    pub default_sample_rate: u32,
    /// 默认声道数
    pub default_channels: u8,
    /// Silero VAD 配置
    pub silero: SileroConfig,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            default_sample_rate: 16000,
            default_channels: 1,
            silero: SileroConfig::default(),
        }
    }
}

/// 音频模块
#[derive(Clone)]
pub struct AudioModule {
    /// Silero VAD 实例
    silero: Silero,
}

impl AudioModule {
    /// 从配置文件初始化
    pub async fn init_from_file_config() -> Result<Self> {
        Self::init(nihility_config::get_config::<AudioConfig>(env!(
            "CARGO_PKG_NAME"
        ))?)
        .await
    }

    /// 直接初始化
    pub async fn init(config: AudioConfig) -> Result<Self> {
        Ok(Self {
            silero: Silero::init(config.silero)?,
        })
    }

    /// 将 PCM 数据转换为 WAV 格式
    pub fn pcm_to_wav(&self, param: PcmToWavParam) -> Result<Vec<u8>> {
        func::pcm_to_wav::pcm_to_wav(param)
    }

    /// 将多声道音频数据合并为单声道
    pub fn merge_channels(&self, param: MergeChannelsParam) -> Result<Vec<f32>> {
        func::merge_channels::merge_channels(param)
    }

    /// 创建 VAD 流式识别器
    ///
    /// 每次调用创建新的处理器，拥有独立的 VAD 状态，但共享 Silero session
    pub fn create_vad_stream_handler(&self, param: VadStreamParam) -> Result<VadStreamHandler> {
        Ok(VadStreamHandler::new(self.silero.clone(), param))
    }
}
