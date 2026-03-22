use crate::error::Result;
use hound::{WavSpec, WavWriter};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

/// 将 PCM 数据转换为 WAV 格式的参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PcmToWavParam {
    /// f32 归一化音频数据 (32-bit float PCM)
    pub pcm_data: Vec<f32>,
    /// 采样率 (如 16000, 44100)
    pub sample_rate: u32,
    /// 声道数 (1 为单声道, 2 为立体声)
    pub channels: u8,
}

/// 将 PCM 数据转换为 WAV 格式
pub fn pcm_to_wav(
    PcmToWavParam {
        pcm_data,
        sample_rate,
        channels,
    }: PcmToWavParam,
) -> Result<Vec<u8>> {
    let spec = WavSpec {
        channels: channels as u16,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut buffer = Vec::new();
    {
        let mut writer = WavWriter::new(Cursor::new(&mut buffer), spec)?;
        for &sample in &pcm_data {
            writer.write_sample(sample)?;
        }
        writer.finalize()?;
    }

    Ok(buffer)
}
