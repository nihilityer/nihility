use crate::device::Device;
use crate::error::*;
use axum::extract::ws::Message as WsMessage;
use axum::extract::ws::WebSocket;
use futures::{SinkExt, StreamExt};
use nihility_edge_protocol::Message;
use postcard::{from_bytes, to_allocvec};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

/// 采样率 16000Hz，10秒 = 160000 采样点
const SAMPLES_PER_10SEC: usize = 16000 * 10;

/// 音频缓冲状态
struct AudioBuffer {
    buffer: Vec<f32>,
    start_timestamp: Option<u64>,
    file_index: u32,
}

impl AudioBuffer {
    fn new() -> Self {
        Self {
            buffer: Vec::new(),
            start_timestamp: None,
            file_index: 0,
        }
    }

    fn push(
        &mut self,
        data: Vec<f32>,
        device_id: &str,
        output_dir: &PathBuf,
    ) -> Result<Option<PathBuf>> {
        // 如果缓冲为空，记录开始时间戳
        if self.buffer.is_empty() {
            self.start_timestamp = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
            self.file_index = 0;
            info!("Start audio recording for device '{}'", device_id);
        }

        self.buffer.extend(data);

        // 检查是否达到10秒
        if self.buffer.len() >= SAMPLES_PER_10SEC {
            let timestamp = self.start_timestamp.unwrap();
            let file_name = format!("{}_{}_{}.wav", device_id, timestamp, self.file_index);
            let file_path = output_dir.join(&file_name);

            write_wav_file(&file_path, &self.buffer[..SAMPLES_PER_10SEC])?;

            info!(
                "Audio file written: {} ({} samples)",
                file_path.display(),
                SAMPLES_PER_10SEC
            );

            // 保留剩余数据
            let remaining: Vec<f32> = self.buffer[SAMPLES_PER_10SEC..].to_vec();
            self.buffer = remaining;
            self.file_index += 1;

            // 更新开始时间戳为新的段开始时间
            self.start_timestamp = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );

            return Ok(Some(file_path));
        }
        Ok(None)
    }
}

pub(crate) async fn start_message_handle(
    web_socket: WebSocket,
    device_id: String,
    devices: Arc<RwLock<HashMap<String, Device>>>,
    cancellation_token: CancellationToken,
) -> Result<mpsc::UnboundedSender<Message>> {
    let (ws_sender, mut ws_receiver) = mpsc::unbounded_channel();
    let (mut ws_sink, mut ws_stream) = web_socket.split();

    // 创建音频缓冲和输出目录
    let mut audio_buffer = AudioBuffer::new();
    let audio_output_dir = PathBuf::from("output/audio");
    // 确保输出目录存在
    std::fs::create_dir_all(audio_output_dir.as_path()).ok();

    // 发送消息到设备
    let send_to_ws_cancellation_token = cancellation_token.clone();
    tokio::spawn(async move {
        while let Some(message) = ws_receiver.recv().await {
            let data = match to_allocvec(&message) {
                Ok(data) => data,
                Err(e) => {
                    error!("Failed to serialize the websocket message: {}", e);
                    send_to_ws_cancellation_token.cancel();
                    break;
                }
            };
            debug!("Send to device, message len: {}", data.len());
            if let Err(e) = ws_sink.send(WsMessage::Binary(data.to_vec().into())).await {
                error!("WebSocket send error: {}", e);
                send_to_ws_cancellation_token.cancel();
                break;
            }
        }
    });

    // 处理来自设备的消息
    tokio::spawn(async move {
        while let Some(msg_result) = ws_stream.next().await {
            match msg_result {
                Ok(WsMessage::Binary(data)) => match from_bytes::<Message>(&data) {
                    Ok(msg) => match msg {
                        Message::KeyEvent(key_event) => {
                            let devices = devices.read().await;
                            if let Some(device) = devices.get(&device_id)
                                && let Some(key_sender) = device.key_sender.clone()
                                && let Err(e) = key_sender.send(key_event.key_code)
                            {
                                error!("Error sending key event: {}", e);
                            }
                        }
                        Message::AudioData(audio_data) => {
                            match audio_buffer.push(
                                audio_data.audio_data,
                                &device_id,
                                &audio_output_dir,
                            ) {
                                Ok(Some(path)) => {
                                    debug!("Audio segment saved to {:?}", path);
                                }
                                Ok(None) => {}
                                Err(e) => {
                                    error!("Audio buffer error: {}", e);
                                }
                            }
                        }
                        _ => {
                            warn!("Received unexpected message: {:?}", msg);
                        }
                    },
                    Err(e) => error!("Failed to deserialize the websocket message: {}", e),
                },
                Ok(_) => warn!("Unsupported message"),
                Err(e) => {
                    error!("WebSocket receive error: {}", e);
                    break;
                }
            }
        }
        cancellation_token.cancel();
        Result::<()>::Ok(())
    });

    Ok(ws_sender)
}

/// 使用 hound 库写入 WAV 文件
/// - 采样率: 16000Hz
/// - 通道数: 1 (单通道)
/// - 位深: 16bit
fn write_wav_file(file_path: &PathBuf, pcm_data: &[f32]) -> Result<()> {
    return Ok(());
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 16000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(file_path, spec)?;
    for &sample in pcm_data {
        writer.write_sample(sample)?;
    }
    writer.finalize()?;
    Ok(())
}
