use crate::error::{AudioError, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 声道合并参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MergeChannelsParam {
    /// f32 归一化音频数据
    pub waveform: Vec<f32>,
    /// 原始声道数 (1 或 2)
    pub channels: u8,
}

/// 将多声道音频数据合并为单声道
/// 对于立体声 (2声道)，将左右声道取平均
pub fn merge_channels(
    MergeChannelsParam { waveform, channels }: MergeChannelsParam,
) -> Result<Vec<f32>> {
    if channels == 1 {
        return Ok(waveform);
    }

    if channels != 2 {
        return Err(AudioError::InvalidChannelCount(channels));
    }

    let num_samples = waveform.len() / 2;
    let mut mono = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let left = waveform[i * 2];
        let right = waveform[i * 2 + 1];
        mono.push((left + right) / 2.0);
    }

    Ok(mono)
}
