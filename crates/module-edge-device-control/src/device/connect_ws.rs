use crate::device::Device;
use crate::error::*;
use futures::{SinkExt, StreamExt};
use nihility_edge_protocol::Message;
use nihility_module_browser_control::func::press_key::PressKeyParam;
use nihility_module_browser_control::BrowserControl;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

pub async fn connect_ws<S: Into<SocketAddr>>(
    addr: S,
    device_id: String,
    devices: Arc<RwLock<HashMap<String, Device>>>,
    browser_control: Arc<RwLock<BrowserControl>>,
    cancellation_token: CancellationToken,
) -> Result<mpsc::UnboundedSender<Message>> {
    let addr = addr.into();
    let url = format!("ws://{}", addr);
    let (ws_stream, _) = connect_async(&url).await?;
    info!("WebSocket connected to {}", addr);

    let (mut ws_sink, mut ws_stream) = ws_stream.split();

    let (tx_to_device, mut rx_from_app) = mpsc::unbounded_channel::<Message>();

    let cancellation_token = cancellation_token.clone();

    // 发送任务（应用 → 设备）
    let send_cancellation_token = cancellation_token.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx_from_app.recv().await {
            let data = match rkyv::to_bytes::<rkyv::rancor::Error>(&msg) {
                Ok(data) => data,
                Err(e) => {
                    error!("Failed to serialize the websocket message: {}", e);
                    break;
                }
            };
            debug!("Send to device, message len: {}", data.len());
            if let Err(e) = ws_sink.send(WsMessage::Binary(data.to_vec().into())).await {
                error!("WebSocket send error: {}", e);
                break;
            }
        }
        send_cancellation_token.cancel();
    });

    // 接收任务（设备 → 应用）
    let recv_cancellation_token = cancellation_token.clone();
    tokio::spawn(async move {
        while let Some(msg_result) = ws_stream.next().await {
            match msg_result {
                Ok(WsMessage::Binary(data)) => {
                    match rkyv::from_bytes::<Message, rkyv::rancor::Error>(&data) {
                        Ok(msg) => match msg {
                            Message::KeyEvent(key_event) => {
                                let devices = devices.read().await;
                                if let Some(device) = devices.get(&device_id)
                                    && let Some(page_id) = device.page_id.clone()
                                {
                                    let key = key_event.key_code.to_browser_key();
                                    browser_control
                                        .write()
                                        .await
                                        .press_key(PressKeyParam { page_id, key })
                                        .await?;
                                }
                            }
                            Message::AudioData(_audio_data) => {
                                warn!("Audio data received from device: {}", device_id);
                            }
                            _ => {
                                warn!("unhandled message {:?}", msg);
                            }
                        },
                        Err(e) => error!("Failed to deserialize message: {}", e),
                    }
                }
                Err(e) => {
                    error!("WebSocket receive error: {}", e);
                    break;
                }
                _ => {}
            }
        }
        // 连接断开，取消所有相关任务
        info!("WebSocket disconnected, cancelling tasks");
        recv_cancellation_token.cancel();
        Result::<()>::Ok(())
    });

    Ok(tx_to_device)
}
