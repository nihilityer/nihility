//! 设备音频处理任务
//!
//! 负责处理来自设备的音频数据，执行 VAD 检测和 ASR 识别

use crate::error::*;
use crate::AsrResult;
use nihility_edge_protocol::AudioData;
use nihility_module_audio::AudioModule;
use nihility_module_audio::VadStreamParam;
use nihility_module_model::ModelModule;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};

/// 音频缓冲区元素
#[derive(Debug, Clone)]
struct AudioChunk {
    /// 音频数据
    data: Vec<f32>,
}

/// 音频处理器
pub struct AudioHandler {
    /// 设备ID
    device_id: String,
    /// 音频模块
    audio_module: Arc<AudioModule>,
    /// 模型模块
    model_module: Arc<RwLock<ModelModule>>,
    /// 音频数据缓冲
    audio_buffer: VecDeque<AudioChunk>,
    /// 当前活动的语音片段索引
    current_speech_segment: Option<u32>,
    /// VAD chunk 计数器
    vad_chunk_counter: usize,
    /// ASR 结果回调
    asr_result_tx: Arc<crate::broadcast::Sender<AsrResult>>,
}

impl AudioHandler {
    /// 创建新的音频处理器
    pub fn new(
        device_id: String,
        audio_module: Arc<AudioModule>,
        model_module: Arc<RwLock<ModelModule>>,
        asr_result_tx: Arc<crate::broadcast::Sender<AsrResult>>,
    ) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(Self {
            device_id,
            audio_module,
            model_module,
            audio_buffer: VecDeque::new(),
            current_speech_segment: None,
            vad_chunk_counter: 0,
            asr_result_tx,
        }))
    }

    /// 处理接收到的音频数据
    ///
    /// 注意：音频采样率固定为 16000 Hz
    pub async fn process_audio(&mut self, audio_data: AudioData) -> Result<()> {
        let AudioData {
            audio_data: samples,
            sample_rate: _sample_rate,
            channels,
            timestamp: _audio_timestamp,
        } = audio_data;

        // 验证采样率（固定 16000）
        // 如果需要支持其他采样率，可以在这里添加重采样逻辑
        debug_assert_eq!(_sample_rate, 16000, "Audio sample rate must be 16000 Hz");

        // 如果是立体声，先合并为单声道
        let samples = if channels > 1 {
            self.audio_module
                .merge_channels(nihility_module_audio::MergeChannelsParam {
                    waveform: samples,
                    channels,
                })?
        } else {
            samples
        };

        // 将音频数据添加到缓冲区
        let chunk = AudioChunk { data: samples };
        self.audio_buffer.push_back(chunk);

        // 保持缓冲区大小，防止内存泄漏
        // 保留足够的音频数据以应对 VAD 的延迟
        while self.audio_buffer.len() > 100 {
            self.audio_buffer.pop_front();
        }

        Ok(())
    }

    /// 通知语音片段开始
    pub fn on_speech_start(&mut self, segment_index: u32) {
        debug!(
            "Speech start detected for segment {} on device {}",
            segment_index, self.device_id
        );
        self.current_speech_segment = Some(segment_index);
    }

    /// 通知语音片段结束，提取音频并进行 ASR 识别
    pub async fn on_speech_end(&mut self, segment_index: u32) -> Result<()> {
        debug!(
            "Speech end detected for segment {} on device {}",
            segment_index, self.device_id
        );

        // 检查是否是当前活动的片段
        if self.current_speech_segment != Some(segment_index) {
            warn!(
                "Received speech end for segment {} but current is {:?}",
                segment_index, self.current_speech_segment
            );
            return Ok(());
        }

        // 收集属于这个片段的音频数据
        // 由于 VAD 的延迟，我们需要收集从语音开始到现在的所有音频
        // 这里简化处理，收集所有缓冲区中的音频
        let mut speech_audio = Vec::new();

        for chunk in self.audio_buffer.iter() {
            // 简单策略：收集最近的一部分音频作为语音片段
            // 实际应用中可能需要更精确的逻辑
            speech_audio.extend_from_slice(&chunk.data);
        }

        // 清空已处理的音频
        self.audio_buffer.clear();

        // 如果音频太短，跳过识别
        if speech_audio.len() < 1600 {
            // 小于 100ms 的音频
            info!("Speech audio too short, skipping ASR");
            return Ok(());
        }

        // 调用 ASR 进行识别
        let result = {
            let model = self.model_module.write().await;
            model
                .speech_recognition(
                    nihility_module_model::func::speech_recognition::SpeechRecognitionParam {
                        audio_data: speech_audio,
                        sample_rate: 16000,
                        channels: 1,
                    },
                )
                .await
        };

        match result {
            Ok(text) if !text.is_empty() => {
                info!(
                    "ASR result for device {} segment {}: {}",
                    self.device_id, segment_index, text
                );

                // 广播识别结果
                let asr_result = AsrResult {
                    device_id: self.device_id.clone(),
                    text,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                };
                let _ = self.asr_result_tx.send(asr_result);
            }
            Ok(_) => {
                debug!("ASR returned empty result for device {}", self.device_id);
            }
            Err(e) => {
                error!("ASR error for device {}: {:?}", self.device_id, e);
            }
        }

        Ok(())
    }
}

/// 启动音频处理任务
///
/// 该任务负责：
/// 1. 从音频 channel 接收设备发送的音频数据
/// 2. 进行 VAD 检测
/// 3. 检测到语音时，收集音频片段
/// 4. 语音结束后进行 ASR 识别
/// 5. 通过 broadcast 发送识别结果
pub async fn start_audio_handler(
    device_id: String,
    audio_module: Arc<AudioModule>,
    model_module: Arc<RwLock<ModelModule>>,
    asr_result_tx: Arc<crate::broadcast::Sender<AsrResult>>,
    mut audio_rx: mpsc::Receiver<AudioData>,
) -> Result<()> {
    let handler = AudioHandler::new(
        device_id.clone(),
        audio_module.clone(),
        model_module,
        asr_result_tx,
    );

    // 创建 VAD 流式处理器
    let mut vad_handler = audio_module
        .create_vad_stream_handler(VadStreamParam::default())
        .map_err(|e| EdgeDeviceControlError::Other(format!("VAD init error: {}", e)))?;

    // 处理音频数据
    tokio::spawn(async move {
        while let Some(audio_data) = audio_rx.recv().await {
            {
                let mut handler = handler.write().await;
                if let Err(e) = handler.process_audio(audio_data).await {
                    error!("Error processing audio: {:?}", e);
                    continue;
                }
            }

            // 获取处理后的音频并发送到 VAD 检测
            let audio_chunk = {
                let handler = handler.read().await;
                handler.audio_buffer.back().cloned()
            };

            if let Some(chunk) = audio_chunk {
                // 使用新的 process 方法进行 VAD 检测
                match vad_handler.process(&chunk.data).await {
                    Ok(Some(segment)) => {
                        let mut handler = handler.write().await;
                        handler.on_speech_start(segment.index);
                    }
                    Ok(None) => {
                        // 没有检测到语音片段，继续
                    }
                    Err(e) => {
                        error!("VAD process error: {:?}", e);
                    }
                }
            }
        }
    });

    Ok(())
}
