use crate::device::screen_processor::{ScreenProcessor, ScreenUpdate};
use crate::device::{Device, DeviceInfo};
use crate::error::*;
use nihility_edge_protocol::Message;
use nihility_module_browser_control::func::screenshot::ScreenshotParam;
use nihility_module_browser_control::BrowserControl;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};
use uuid::Uuid;

/// 新建一个线程处理设备屏幕刷新推送
pub(crate) async fn start_screen_refresh(
    device_info: DeviceInfo,
    devices: Arc<RwLock<HashMap<String, Device>>>,
    browser_control: Arc<RwLock<BrowserControl>>,
    page_id: Uuid,
    screenshot_selector: Option<String>,
    cancellation_token: CancellationToken,
) -> Result<JoinHandle<Result<()>>> {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(
        device_info.screen_refresh_interval as u64,
    ));
    info!(
        "Screen refresh task started for device {} (interval: {}ms)",
        device_info.device_id, device_info.screen_refresh_interval
    );

    let mut processor = ScreenProcessor::new(
        device_info.screen_width,
        device_info.screen_height,
        device_info.screen_config,
    );

    let screenshot_param = ScreenshotParam {
        page_id: page_id.to_string(),
        selector: screenshot_selector.clone(),
    };
    let join_handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // 正常 tick，继续处理
                }
                _ = cancellation_token.cancelled() => {
                    info!("Screen refresh task for device {} cancelled", device_info.device_id);
                    break;
                }
            }

            let png_data = browser_control
                .read()
                .await
                .screenshot(screenshot_param.clone())
                .await?;

            // 转换为 1-bit 位图
            let full_screen = match processor.convert_png_to_1bit(&png_data) {
                Ok(screen) => screen,
                Err(e) => {
                    error!("Failed to convert screenshot: {}", e);
                    continue;
                }
            };

            // 计算更新类型
            // 根据更新类型决定是否发送消息
            match processor.diff(full_screen) {
                ScreenUpdate::Full(full_screen) => {
                    info!("Full screen update -> {}", device_info.device_id);
                    let msg = Message::FullScreenUpdate(full_screen);

                    let devices_guard = devices.read().await;
                    let device = devices_guard.get(&device_info.device_id).ok_or_else(|| {
                        EdgeDeviceControlError::DeviceStatus(format!(
                            "device {} not found",
                            device_info.device_id
                        ))
                    })?;
                    let ws_sender = device.ws_sender.as_ref().ok_or_else(|| {
                        EdgeDeviceControlError::DeviceStatus(format!(
                            "device {} not connecting",
                            device_info.device_id
                        ))
                    })?;
                    ws_sender.send(msg).map_err(|_| {
                        EdgeDeviceControlError::DeviceStatus(format!(
                            "device {} ws_sender failed to send",
                            device_info.device_id
                        ))
                    })?;
                }
                ScreenUpdate::Incremental(incremental) => {
                    info!("Incremental update -> {}", device_info.device_id);
                    let msg = Message::IncrementalScreenUpdate(incremental);

                    let devices_guard = devices.read().await;
                    let device = devices_guard.get(&device_info.device_id).ok_or_else(|| {
                        EdgeDeviceControlError::DeviceStatus(format!(
                            "device {} not found",
                            device_info.device_id
                        ))
                    })?;
                    let ws_sender = device.ws_sender.as_ref().ok_or_else(|| {
                        EdgeDeviceControlError::DeviceStatus(format!(
                            "device {} not connecting",
                            device_info.device_id
                        ))
                    })?;
                    ws_sender.send(msg).map_err(|_| {
                        EdgeDeviceControlError::DeviceStatus(format!(
                            "device {} ws_sender failed to send",
                            device_info.device_id
                        ))
                    })?;
                }
                ScreenUpdate::Skip => {}
            }
        }
        cancellation_token.cancel();
        Ok(())
    });
    Ok(join_handle)
}
