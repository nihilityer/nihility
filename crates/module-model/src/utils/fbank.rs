use ndarray::{Array1, Array2, Axis, stack};
use num_complex::Complex;
use realfft::{RealFftPlanner, RealToComplex};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::error;

/// 实时fbank特征提取
pub struct OnlineFbank {
    frame_len: usize,
    frame_shift: usize,
    fft_size: usize,
    mel_filters: Array2<f32>,
    real_fft: Arc<dyn RealToComplex<f32>>,
    scratch_input: Vec<f32>,
    scratch_spectrum: Vec<Complex<f32>>,
    buffer: Vec<f32>,
    window_type: WindowType,
}

/// 实时fbank特征提取初始化配置
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct OnlineFbankConfig {
    pub sample_rate: usize,
    pub frame_len_ms: usize,
    pub frame_shift_ms: usize,
    pub num_mel_bins: usize,
    pub low_freq: f64,
    pub high_freq: f64,
    pub window_type: WindowType,
}

/// 实时fbank特征提取加窗类型
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub enum WindowType {
    Hanning,
    Hamming,
}

impl OnlineFbank {
    /// 根据配置初始化在线fbank提取
    pub fn init(config: OnlineFbankConfig) -> Self {
        let frame_len =
            ((config.frame_len_ms as f64 / 1000.0) * config.sample_rate as f64) as usize;
        let frame_shift =
            ((config.frame_shift_ms as f64 / 1000.0) * config.sample_rate as f64) as usize;

        let fft_size = frame_len.next_power_of_two();

        // 生成 Mel 滤波器组
        let mel_filters = mel_filter_bank(
            fft_size / 2 + 1,
            config.sample_rate,
            config.num_mel_bins,
            config.low_freq,
            config.high_freq,
        );

        let mut real_planner = RealFftPlanner::new();
        let real_fft = real_planner.plan_fft_forward(fft_size);

        Self {
            frame_len,
            frame_shift,
            fft_size,
            mel_filters,
            real_fft,
            scratch_input: vec![0.0f32; fft_size],
            scratch_spectrum: vec![Complex::new(0.0, 0.0); fft_size / 2 + 1],
            buffer: Vec::new(),
            window_type: WindowType::Hanning,
        }
    }

    /// 清空缓冲区，当音频数据结束准备接受下一段音频数据时调用
    pub fn buffer_clear(&mut self) {
        self.buffer.clear();
    }

    /// 传入一段音频数据
    pub fn accept_waveform(&mut self, mut samples: Vec<f32>) {
        self.buffer.append(&mut samples)
    }

    /// 返回当前未返回的所有音频帧fbank特征
    pub fn process_all(&mut self) -> Array2<f32> {
        let features: Vec<Array1<f32>> = std::iter::from_fn(|| self.next_frame()).collect();

        if features.is_empty() {
            Array2::zeros((0, 0))
        } else {
            stack(
                Axis(0),
                &features.iter().map(|x| x.view()).collect::<Vec<_>>(),
            )
            .unwrap_or_else(|_| Array2::zeros((0, 0)))
        }
    }

    /// 返回下一个音频帧的fbank特征
    pub fn next_frame(&mut self) -> Option<Array1<f32>> {
        if self.buffer.len() >= self.frame_len {
            let frame = &self.buffer[..self.frame_len];

            // 1. 加窗
            let windowed: Vec<f32> = match self.window_type {
                WindowType::Hanning => (0..self.frame_len)
                    .map(|i| {
                        let w = 0.5
                            * (1.0
                                - (2.0 * std::f32::consts::PI * i as f32
                                    / (self.frame_len - 1) as f32)
                                    .cos());
                        frame[i] * w
                    })
                    .collect(),
                WindowType::Hamming => (0..self.frame_len)
                    .map(|i| {
                        let w = 0.54
                            - 0.46
                                * (2.0 * std::f32::consts::PI * i as f32
                                    / (self.frame_len - 1) as f32)
                                    .cos();
                        frame[i] * w
                    })
                    .collect(),
            };

            // 2. 计算功率谱
            let power_spectrum = self.compute_power_spectrum(&windowed);

            // 3. Mel 滤波器组 + log
            let mel_energy = self.mel_filters.dot(&power_spectrum);
            let log_mel = mel_energy.mapv(|x| (x + 1e-10f32).ln());

            // 滑动窗口
            self.buffer.drain(..self.frame_shift);

            Some(log_mel)
        } else {
            None
        }
    }

    fn compute_power_spectrum(&mut self, windowed: &[f32]) -> Array1<f32> {
        let input = &mut self.scratch_input;
        let spectrum = &mut self.scratch_spectrum;

        // 1. 清零 + 复制窗口数据
        input.resize(self.fft_size, 0.0f32);
        input[..windowed.len()].copy_from_slice(windowed);

        // 2. 执行实数 FFT
        if let Err(e) = self.real_fft.process(input, spectrum) {
            error!(
                "exec fft error, input: {:?}, output: {:?}, message: {}",
                input, spectrum, e
            )
        }

        // 3. 计算功率谱：|X|^2 / fft_size
        let norm = 1.0f32 / self.fft_size as f32;

        Array1::from_iter(spectrum.iter().map(|c| c.norm_sqr() * norm))
    }
}

// 生成和 librosa / kaldi 兼容的 Mel 滤波器组（HTK 公式）
fn mel_filter_bank(
    num_fft_bins: usize,
    sample_rate: usize,
    num_mel_bins: usize,
    low_freq: f64,
    high_freq: f64,
) -> Array2<f32> {
    let fft_freqs = Array1::linspace(0.0, sample_rate as f64 / 2.0, num_fft_bins);
    let min_mel = hz_to_mel(low_freq);
    let max_mel = hz_to_mel(high_freq);
    let mel_points = Array1::linspace(min_mel, max_mel, num_mel_bins + 2);

    let hz_points = mel_points.mapv(mel_to_hz);

    let mut filters = Array2::<f32>::zeros((num_mel_bins, num_fft_bins));

    for m in 0..num_mel_bins {
        let left = hz_points[m];
        let center = hz_points[m + 1];
        let right = hz_points[m + 2];

        for (j, freq) in fft_freqs.iter().enumerate() {
            let freq = *freq;
            let val = if freq >= left && freq < center {
                (freq - left) / (center - left)
            } else if freq >= center && freq < right {
                (right - freq) / (right - center)
            } else {
                0.0
            };
            filters[[m, j]] = val as f32;
        }
    }

    // 归一化
    let mut enorm = Array1::<f32>::zeros(num_mel_bins);
    for m in 0..num_mel_bins {
        let row = filters.row(m);
        enorm[m] = 2.0 / (hz_points[m + 2] - hz_points[m]) as f32 * row.sum();
    }
    for m in 0..num_mel_bins {
        let inv = 1.0 / (enorm[m] + 1e-12);
        filters.row_mut(m).mapv_inplace(|x| x * inv);
    }

    filters
}

fn hz_to_mel(freq: f64) -> f64 {
    2595.0 * (1.0 + freq / 700.0).log10()
}

fn mel_to_hz(mel: f64) -> f64 {
    700.0 * (10f64.powf(mel / 2595.0) - 1.0)
}

impl Default for OnlineFbankConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            frame_len_ms: 25,
            frame_shift_ms: 10,
            num_mel_bins: 80,
            low_freq: 20.0,
            high_freq: 8000.0,
            window_type: WindowType::Hanning,
        }
    }
}
