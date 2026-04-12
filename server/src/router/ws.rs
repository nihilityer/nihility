use crate::error::*;
use crate::router::not_found;
use crate::AppState;
use axum::extract::{State, WebSocketUpgrade};
use axum::response::Response;
use axum::routing::any;
use axum::Router;
use tracing::{debug, error};

pub fn ws_router() -> Router<AppState> {
    Router::new()
        .route("/edge-device-control", any(edge_device_control))
        .fallback(not_found)
}

async fn edge_device_control(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Result<Response> {
    debug!("WS edge device control");
    let edge_device_control = state.module_manager.get_edge_device_control()?;
    let web_socket_sender = edge_device_control
        .read()
        .await
        .get_web_socket_sender()
        .map_err(|err| {
            error!(%err);
            NihilityServerError::NotFound(format!("EdgeDeviceControl; error: {}", err))
        })?;
    Ok(ws.on_upgrade(|socket| async move {
        if let Err(e) = web_socket_sender.send(socket) {
            error!("failed to send websocket: {}", e);
        }
    }))
}
