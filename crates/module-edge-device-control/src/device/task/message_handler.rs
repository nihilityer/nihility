use crate::device::Device;
use crate::error::*;
use nihility_edge_protocol::Message;
use nihility_module_browser_control::func::press_key::PressKeyParam;
use nihility_module_browser_control::BrowserControl;
use nihility_module_model::ModelModule;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

/// 处理设备消息（按键事件、音频数据等）
pub(crate) async fn start_message_handler(
    device_id: String,
    devices: Arc<RwLock<HashMap<String, Device>>>,
    browser_control: Arc<RwLock<BrowserControl>>,
    _model_module: Arc<RwLock<ModelModule>>,
    _asr_result_tx: Arc<broadcast::Sender<crate::AsrResult>>,
    mut rx: mpsc::UnboundedReceiver<Message>,
    cancellation_token: CancellationToken,
) -> Result<()> {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                msg = rx.recv() => {
                    match msg {
                        Some(msg) => {
                            match msg {
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
                            }
                        }
                        None => {
                            cancellation_token.cancel();
                            break
                        }, // channel 关闭
                    }
                }
                _ = cancellation_token.cancelled() => {
                    info!("Message handler for device {} cancelled", device_id);
                    break;
                }
            }
        }
        info!("Message handler for device {} exiting", device_id);
        Result::<()>::Ok(())
    });
    Ok(())
}
