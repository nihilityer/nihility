use crate::device::task::audio_handle::start_audio_handle;
use crate::device::{start_message_handle, Device};
use crate::error::*;
use axum::extract::ws::{Message, WebSocket};
use nihility_util_vad::{start_vad, VoiceActivityDetectionConfig};
use postcard::from_bytes;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error};

pub(crate) async fn register_device(
    mut web_socket: WebSocket,
    devices: Arc<RwLock<HashMap<String, Device>>>,
) -> Result<()> {
    let mut device = None;
    // 接受来自设备的信息后注册新设备
    while let Some(Ok(msg)) = web_socket.recv().await {
        match msg {
            Message::Binary(bytes) => {
                debug!("received Binary message: {:?}", bytes);
                match from_bytes::<nihility_edge_protocol::Message>(&bytes) {
                    Ok(msg) => match msg {
                        nihility_edge_protocol::Message::DeviceInfo(device_info) => {
                            device = Some(Device::new(device_info));
                            break;
                        }
                        _ => {
                            error!("received invalid device info message");
                            break;
                        }
                    },
                    Err(e) => {
                        error!("Failed to deserialize message: {}", e);
                        break;
                    }
                }
            }
            Message::Ping(a) => web_socket.send(Message::Pong(a)).await.map_err(|e| {
                EdgeDeviceControlError::DeviceStatus(format!("send pong error: {}", e))
            })?,
            Message::Close(_) => break,
            _ => {}
        }
    }
    if let Some(mut device) = device {
        debug!("register device: {}", device.info.device_id);
        let config = nihility_config::get_config::<VoiceActivityDetectionConfig>(&format!(
            "device_{}_vad",
            device.info.device_id
        ))?;
        let (sample_sender, sample_receiver) = mpsc::unbounded_channel();
        let (speech_receiver, vad_join_handle) = start_vad(config, sample_receiver).await?;
        let audio_handle_task =
            start_audio_handle(device.info.device_id.clone(), speech_receiver).await?;
        device.audio_vad_task = Some(vad_join_handle);
        device.audio_handle_task = Some(audio_handle_task);

        let ws_sender = start_message_handle(
            web_socket,
            device.info.device_id.clone(),
            devices.clone(),
            device.cancellation_token.clone(),
            sample_sender,
        )
        .await?;
        device.ws_sender = Some(ws_sender);
        devices
            .write()
            .await
            .insert(device.info.device_id.clone(), device);
    }
    Ok(())
}
