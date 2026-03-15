use crate::device::screen_processor::ScreenProcessor;
use crate::device::Device;
use crate::error::*;
use nihility_edge_protocol::{DeviceInfo, Message};
use nihility_module_browser_control::func::open_page::OpenPageParam;
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
    mapping_url: &str,
    screenshot_selector: Option<String>,
) -> Result<JoinHandle<Result<()>>> {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(
        device_info.screen_refresh_interval as u64,
    ));
    info!(
        "Started screenshot task for device {} with interval {}ms",
        device_info.device_id, device_info.screen_refresh_interval
    );
    let page_id = browser_control
        .write()
        .await
        .open_page(OpenPageParam {
            url: mapping_url.to_string(),
        })
        .await?;
    let mut processor = ScreenProcessor::new(device_info.screen_width, device_info.screen_height);

    {
        let mut devices_guard = devices.write().await;
        let device = devices_guard
            .get_mut(&device_info.device_id)
            .ok_or_else(|| {
                EdgeDeviceControlError::DeviceStatus(format!(
                    "device {} not found",
                    device_info.device_id
                ))
            })?;
        device.page_id = Some(page_id.clone());
    }

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
                .screenshot(ScreenshotParam {
                    page_id: page_id.clone(),
                    selector: screenshot_selector.clone(),
                })
                .await?;

            // 3. 转换为 1-bit 位图
            let full_screen = match processor.convert_png_to_1bit(&png_data) {
                Ok(screen) => screen,
                Err(e) => {
                    error!("Failed to convert screenshot: {}", e);
                    continue;
                }
            };

            // 4. 计算增量更新
            let msg = match processor.diff(&full_screen) {
                Some(incremental) => Message::IncrementalScreenUpdate(incremental),
                None => Message::FullScreenUpdate(full_screen),
            };

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
    });
    Ok(join_handle)
}
