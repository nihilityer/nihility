use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// 音频数据消息（设备 -> 服务器）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioData {
    /// PCM 32bit 浮点音频数据 (IEEE 754)
    pub audio_data: Vec<f32>,
    /// 采样率 (固定 16000)
    pub sample_rate: u32,
    /// 声道数 (可自定义)
    pub channels: u8,
    /// 时间戳（毫秒）
    pub timestamp: u64,
}
