use crate::device::Device;
use crate::error::*;
use axum::extract::ws::Message as WsMessage;
use axum::extract::ws::WebSocket;
use futures::{SinkExt, StreamExt};
use nihility_edge_protocol::Message;
use postcard::{from_bytes, to_allocvec};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, warn};

pub(crate) async fn start_message_handle(
    web_socket: WebSocket,
    device_id: String,
    devices: Arc<RwLock<HashMap<String, Device>>>,
    cancellation_token: CancellationToken,
) -> Result<mpsc::UnboundedSender<Message>> {
    let (ws_sender, mut ws_receiver) = mpsc::unbounded_channel();
    let (mut ws_sink, mut ws_stream) = web_socket.split();

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
                        Message::AudioData(_) => {
                            warn!("Unimplemented audio data message handle")
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
