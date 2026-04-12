pub mod error;
mod silero;

use crate::error::*;
use crate::silero::{Silero, SileroConfig};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tokio::sync::broadcast::{channel, Receiver};
use tokio::task::JoinHandle;
use tracing::debug;

/// 语音活动检测配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceActivityDetectionConfig {
    pub speech_channel_size: usize,
    pub probability_channel_size: usize,
    pub padding_size: usize,
    pub threshold: f32,
    pub silero_config: SileroConfig,
}

/// 启动 VAD 线程，创建独立的 Silero 实例
///
/// 返回发送到的语音数据接收器
pub async fn start_vad(
    config: VoiceActivityDetectionConfig,
    mut sample_receiver: Receiver<f32>,
) -> Result<(Receiver<Vec<f32>>, JoinHandle<Result<()>>)> {
    debug!("Starting VoiceActivityDetection with config {:?}", &config);
    let (speech_sender, speech_receiver) = channel(config.speech_channel_size);

    let mut silero = Silero::init(config.silero_config.clone())?;

    let join_handle = tokio::spawn(async move {
        let mut buffer =
            VecDeque::with_capacity(silero.config.chunk_size * (config.padding_size + 2));
        let mut cumulative_sample_count: usize = 0;
        let mut silence_count: usize = 0;
        let mut is_speech_active = false;

        while let Ok(sample) = sample_receiver.recv().await {
            buffer.push_back(sample);
            cumulative_sample_count += 1;
            // 当新样本数量积累到一个块大小时进行识别
            if cumulative_sample_count == silero.config.chunk_size {
                // 重置新样本数量计数
                cumulative_sample_count = 0;
                let probability = silero.predict(
                    &buffer
                        .range((buffer.len() - silero.config.chunk_size)..)
                        .copied()
                        .collect::<Vec<f32>>(),
                )?;
                // 当预测结果大于等于设定阈值
                if probability >= config.threshold {
                    is_speech_active = true;
                    silence_count = 0;
                } else {
                    silence_count += 1;
                    if silence_count >= config.padding_size {
                        if is_speech_active {
                            // 如果当前处于活动状态，标志活动语言结束，发送当前缓冲区内所有数据，并且重置缓冲区
                            is_speech_active = false;
                            silence_count = 0;
                            speech_sender.send(buffer.drain(..).collect())?;
                        } else {
                            // 当静音块数量超过设置边界数量，移除最早的音频数据块
                            if silence_count != config.padding_size {
                                silence_count -= 1;
                                buffer.drain(..silero.config.chunk_size);
                            }
                        }
                    }
                }
            }
        }
        Result::Ok(())
    });
    Ok((speech_receiver, join_handle))
}

impl Default for VoiceActivityDetectionConfig {
    fn default() -> Self {
        Self {
            speech_channel_size: 5,
            silero_config: SileroConfig::default(),
            padding_size: 4,
            threshold: 0.01,
            probability_channel_size: 0,
        }
    }
}
