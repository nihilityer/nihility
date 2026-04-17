use crate::device::Device;
use crate::error::*;
use crate::EdgeDeviceControl;
use nihility_module_browser_control::func::close_page::ClosePageParam;
use nihility_module_browser_control::func::open_page::OpenPageParam;
use nihility_module_browser_control::BrowserControl;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

/// 连接新设备
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConnectDeviceParam {
    /// 设备Id
    pub device_id: String,
    /// 需要显示在设备屏幕上的网页Url
    pub mapping_url: String,
    /// 屏幕映射网页中哪个元素
    pub screenshot_selector: Option<String>,
    /// 设备对应的场景Id
    pub scene_id: Uuid,
}

impl EdgeDeviceControl {
    pub async fn connect_device(&mut self, param: ConnectDeviceParam) -> Result<()> {
        if self.browser_control.is_none() {
            return Err(EdgeDeviceControlError::ModuleStatus(
                "browser_control is required".to_string(),
            ));
        }
        connect_device(
            param.scene_id,
            param.device_id,
            param.mapping_url,
            param.screenshot_selector,
            self.devices.clone(),
            self.browser_control.as_ref().unwrap().clone(),
        )
        .await?;
        Ok(())
    }
}

pub async fn connect_device(
    scene_id: Uuid,
    device_id: String,
    mapping_url: String,
    screenshot_selector: Option<String>,
    devices: Arc<RwLock<HashMap<String, Device>>>,
    browser_control: Arc<RwLock<BrowserControl>>,
) -> Result<()> {
    let mut devices_guard = devices.write().await;
    let device = devices_guard.get_mut(&device_id).ok_or_else(|| {
        EdgeDeviceControlError::DeviceStatus(format!("device {} not found", device_id))
    })?;
    if let Some(scene_id_sender) = device.scene_id_sender.take()
        && scene_id_sender.send(scene_id).is_ok()
    {
        debug!(?scene_id, "send scene id to audio handle");
    } else {
        return Err(EdgeDeviceControlError::DeviceStatus(
            "failed to send scene id to audio handle".to_string(),
        ));
    }

    if let Some(task) = &device.screen_refresh_task {
        task.abort();
    }
    if let Some(task) = &device.key_handle_task {
        task.abort();
    }
    if let Some(page_id) = device.page_id {
        browser_control
            .write()
            .await
            .close_page(ClosePageParam { page_id })
            .await?;
    }

    let page_id = browser_control
        .write()
        .await
        .open_page(OpenPageParam {
            url: mapping_url.to_string(),
        })
        .await?;
    device.page_id = Some(page_id);
    info!("connect to device {} with page id: {}", device_id, page_id);
    // 等待几秒让页面加载完成
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    device
        .start_screen_push(
            devices.clone(),
            browser_control.clone(),
            page_id,
            screenshot_selector,
        )
        .await?;
    device
        .start_key_handle(browser_control.clone(), page_id)
        .await?;
    Ok(())
}
