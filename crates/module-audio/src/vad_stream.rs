//! VAD 流式处理模块
//!
//! 提供独立的 VAD 流式处理器，每个实例拥有独立的 Silero 状态

use crate::error::Result;
use crate::vad::silero::Silero;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// VAD 流式识别参数
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VadStreamParam {
    /// 语音概率阈值 (0.0-1.0)，超过此值认为是语音，默认 0.5
    #[serde(default = "default_threshold")]
    pub threshold: f32,
    /// 最小语音持续chunk数，用于过滤噪声，默认 3
    #[serde(default = "default_min_speech_chunks")]
    pub min_speech_chunks: usize,
    /// 语音结束后确认结束的最小静音chunk数，默认 15
    #[serde(default = "default_min_silence_chunks")]
    pub min_silence_chunks: usize,
}

fn default_threshold() -> f32 {
    0.5
}

fn default_min_speech_chunks() -> usize {
    3
}

fn default_min_silence_chunks() -> usize {
    15
}

impl Default for VadStreamParam {
    fn default() -> Self {
        Self {
            threshold: 0.5,
            min_speech_chunks: 3,
            min_silence_chunks: 15,
        }
    }
}

/// 语音片段输出
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SpeechSegment {
    /// 片段编号，从1开始
    pub index: u32,
}

/// VAD 状态机状态
enum VadState {
    /// 等待语音
    Idle,
    /// 检测到语音
    Speech {
        speech_chunks: usize,
        segment_index: u32,
    },
    /// 语音结束，等待确认
    Silence {
        speech_chunks: usize,
        silence_chunks: usize,
        segment_index: u32,
    },
}

/// VAD 流式识别器
///
/// 每个实例拥有独立的 Silero 状态（通过 clone 实现），但共享同一个 session
pub struct VadStreamHandler {
    silero: Silero,
    param: VadStreamParam,
    state: VadState,
    current_segment_index: u32,
}

impl VadStreamHandler {
    /// 创建新的 VAD 流式识别器
    pub fn new(silero: Silero, param: VadStreamParam) -> Self {
        Self {
            silero,
            param,
            state: VadState::Idle,
            current_segment_index: 0,
        }
    }

    /// 处理音频 chunk，返回检测到的语音片段（如果有）
    pub async fn process(&mut self, samples: &[f32]) -> Result<Option<SpeechSegment>> {
        let probability = self.silero.predict(&samples)?;

        let is_speech = probability >= self.param.threshold;

        // 使用 std::mem::replace 避免借用冲突
        let mut state = std::mem::replace(&mut self.state, VadState::Idle);

        let segment = match (&mut state, is_speech) {
            (VadState::Idle, true) => {
                self.current_segment_index += 1;
                state = VadState::Speech {
                    speech_chunks: 1,
                    segment_index: self.current_segment_index,
                };
                if 1 >= self.param.min_speech_chunks {
                    Some(SpeechSegment {
                        index: self.current_segment_index,
                    })
                } else {
                    None
                }
            }
            (VadState::Idle, false) => None,

            (
                VadState::Speech {
                    speech_chunks,
                    segment_index,
                },
                true,
            ) => {
                *speech_chunks += 1;
                if *speech_chunks >= self.param.min_speech_chunks {
                    Some(SpeechSegment {
                        index: *segment_index,
                    })
                } else {
                    None
                }
            }
            (
                VadState::Speech {
                    speech_chunks,
                    segment_index,
                    ..
                },
                false,
            ) => {
                state = VadState::Silence {
                    speech_chunks: *speech_chunks,
                    silence_chunks: 1,
                    segment_index: *segment_index,
                };
                None
            }

            (VadState::Silence { speech_chunks, segment_index, .. }, true) => {
                state = VadState::Speech {
                    speech_chunks: *speech_chunks,
                    segment_index: *segment_index,
                };
                None
            }
            (
                VadState::Silence {
                    silence_chunks,
                    speech_chunks: _,
                    segment_index,
                },
                false,
            ) => {
                *silence_chunks += 1;
                if *silence_chunks >= self.param.min_silence_chunks {
                    let index = *segment_index;
                    state = VadState::Idle;
                    Some(SpeechSegment { index })
                } else {
                    None
                }
            }
        };

        self.state = state;

        Ok(segment)
    }
}

impl Clone for VadStreamHandler {
    fn clone(&self) -> Self {
        // Silero::clone 创建新的 state 但共享 session
        Self::new(self.silero.clone(), self.param.clone())
    }
}
