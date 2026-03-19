use crate::device::screen_processor::{ScreenProcessor, ScreenUpdate};
use crate::device::Device;
use crate::error::*;
use nihility_edge_protocol::{DeviceInfo, Message};
use nihility_module_browser_control::func::screenshot::ScreenshotParam;
use nihility_module_browser_control::BrowserControl;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{error, info};

/// 新建一个线程处理设备屏幕刷新推送
pub(crate) async fn start_screen_refresh(
    device_info: DeviceInfo,
    devices: Arc<RwLock<HashMap<String, Device>>>,
    browser_control: Arc<RwLock<BrowserControl>>,
    page_id: &str,
    screenshot_selector: Option<String>,
) -> Result<JoinHandle<Result<()>>> {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(
        device_info.screen_refresh_interval as u64,
    ));
    info!(
        "Screen refresh task started for device {} (interval: {}ms)",
        device_info.device_id,
        device_info.screen_refresh_interval
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
            interval.tick().await;
            {
                let devices_guard = devices.read().await;
                let device = devices_guard.get(&device_info.device_id).ok_or_else(|| {
                    EdgeDeviceControlError::DeviceStatus(format!(
                        "device {} not found",
                        device_info.device_id
                    ))
                })?;
                if !device.screen_refresh_task_switch {
                    continue;
                }
            }

            let png_data = browser_control
                .read()
                .await
                .screenshot(screenshot_param.clone())
                .await?;

            // 3. 转换为 1-bit 位图
            let full_screen = match processor.convert_png_to_1bit(&png_data) {
                Ok(screen) => screen,
                Err(e) => {
                    error!("Failed to convert screenshot: {}", e);
                    continue;
                }
            };

            // 4. 计算更新类型

            // 5. 根据更新类型决定是否发送消息
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
    });
    Ok(join_handle)
}
