use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// 音频数据消息（设备 -> 服务器）
/// 固定16000Hz采样率，单声道，位深无关
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioData {
    /// PCM 32bit 浮点音频数据
    pub audio_data: Vec<f32>,
    /// 时间戳（毫秒）
    pub timestamp: u64,
}
