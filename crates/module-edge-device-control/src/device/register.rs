use crate::device::task::audio_handle::start_audio_handle;
use crate::device::{start_message_handle, Device};
use crate::error::*;
use crate::func::connect_device;
use axum::extract::ws::{Message, WebSocket};
use nihility_module_browser_control::BrowserControl;
use nihility_module_model::Model;
use nihility_util_vad::{start_vad, VoiceActivityDetectionConfig};
use postcard::from_bytes;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info};

pub(crate) async fn register_device(
    mut web_socket: WebSocket,
    model: Arc<RwLock<Model>>,
    devices: Arc<RwLock<HashMap<String, Device>>>,
    browser_control: Arc<RwLock<BrowserControl>>,
    auto_connect: Arc<HashMap<String, (String, Option<String>)>>,
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
            start_audio_handle(device.info.device_id.clone(), model, speech_receiver).await?;
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
        let device_id = device.info.device_id.clone();
        devices.write().await.insert(device_id.clone(), device);

        if let Some((url, selector)) = auto_connect.get(&device_id) {
            let devices_clone = devices.clone();
            let device_id_clone = device_id.clone();
            let url_clone = url.clone();
            let selector_clone = selector.clone();
            info!(
                "auto-connecting device {} to {}",
                device_id_clone, url_clone
            );
            tokio::spawn(connect_device(
                device_id_clone,
                url_clone,
                selector_clone,
                devices_clone,
                browser_control,
            ));
        }
    }
    Ok(())
}
