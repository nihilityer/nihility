use crate::error::*;
use futures::{SinkExt, StreamExt};
use nihility_edge_protocol::Message;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::{debug, error, info};

pub async fn connect_ws<S: Into<SocketAddr>>(
    addr: S,
) -> Result<(
    mpsc::UnboundedSender<Message>,
    mpsc::UnboundedReceiver<Message>,
)> {
    let addr = addr.into();
    let url = format!("ws://{}", addr);
    let (ws_stream, _) = connect_async(&url).await?;
    info!("WebSocket connected to {}", addr);

    let (mut ws_sink, mut ws_stream) = ws_stream.split();

    let (tx_to_device, mut rx_from_app) = mpsc::unbounded_channel::<Message>();
    let (tx_to_app, rx_to_app) = mpsc::unbounded_channel::<Message>();

    // 发送任务（应用 → 设备）
    tokio::spawn(async move {
        while let Some(msg) = rx_from_app.recv().await {
            let data = match bincode::serialize(&msg) {
                Ok(data) => data,
                Err(e) => {
                    error!("Failed to serialize the websocket message: {}", e);
                    break;
                }
            };
            if let Err(e) = ws_sink.send(WsMessage::Binary(data.into())).await {
                error!("WebSocket send error: {}", e);
                break;
            }
        }
    });

    // 接收任务（设备 → 应用）
    tokio::spawn(async move {
        while let Some(msg_result) = ws_stream.next().await {
            match msg_result {
                Ok(WsMessage::Binary(data)) => match bincode::deserialize::<Message>(&data) {
                    Ok(msg) => {
                        debug!("Received message: {:?}", msg);
                        let _ = tx_to_app.send(msg);
                    }
                    Err(e) => error!("Failed to deserialize message: {}", e),
                },
                Err(e) => {
                    error!("WebSocket receive error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    Ok((tx_to_device, rx_to_app))
}
