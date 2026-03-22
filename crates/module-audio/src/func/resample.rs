use crate::error::{AudioError, Result};
use rubato::{FastFixedIn, PolynomialDegree, VecResampler};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 重采样参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResampleParam {
    /// f32 归一化音频数据
    pub waveform: Vec<f32>,
    /// 原始采样率
    pub from_rate: u32,
    /// 目标采样率
    pub to_rate: u32,
}

/// 将音频采样率转换为目标采样率
pub fn resample(param: ResampleParam) -> Result<Vec<f32>> {
    let ResampleParam {
        waveform,
        from_rate,
        to_rate,
    } = param;

    if from_rate == to_rate {
        return Ok(waveform);
    }

    if from_rate == 0 || to_rate == 0 {
        return Err(AudioError::UnsupportedSampleRate(if from_rate == 0 {
            to_rate
        } else {
            from_rate
        }));
    }

    let ratio = to_rate as f64 / from_rate as f64;
    let mut resampler = FastFixedIn::<f32>::new(
        ratio,
        1.1,
        PolynomialDegree::Cubic,
        1, // nbr_channels - 单声道
        8, // nbr_segments
    )
    .map_err(|e| AudioError::ResampleError(format!("Failed to create resampler: {}", e)))?;

    // process expects &[Vec<f32>] for single channel (interleaved channels)
    let mut input_buffer = vec![waveform];
    let chunks = VecResampler::process(&mut resampler, &mut input_buffer, None)
        .map_err(|e| AudioError::ResampleError(format!("Resampling failed: {}", e)))?;

    Ok(chunks[0].clone())
}
