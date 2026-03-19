use crate::error::*;
use crate::EdgeDeviceControl;
use nihility_module_browser_control::func::open_page::OpenPageParam;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tracing::info;

/// 连接新设备
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConnectDeviceParam {
    /// 设备Id
    pub device_id: String,
    /// 需要显示在设备屏幕上的网页Url
    pub mapping_url: String,
    /// 屏幕映射网页中哪个元素
    pub screenshot_selector: Option<String>,
}

impl EdgeDeviceControl {
    pub async fn connect_device(&mut self, param: ConnectDeviceParam) -> Result<()> {
        if self.browser_control.is_none() {
            return Err(EdgeDeviceControlError::ModuleStatus(
                "browser_control is required".to_string(),
            ));
        }
        let mut devices_guard = self.devices.write().await;
        let device = devices_guard.get_mut(&param.device_id).ok_or_else(|| {
            EdgeDeviceControlError::DeviceStatus(format!("device {} not found", param.device_id))
        })?;
        let page_id = self
            .browser_control
            .as_ref()
            .unwrap()
            .write()
            .await
            .open_page(OpenPageParam {
                url: param.mapping_url.to_string(),
            })
            .await?;
        device.page_id = Some(page_id.clone());
        info!(
            "connect to device {} with page id: {}",
            param.device_id, page_id
        );

        device
            .connect(
                self.devices.clone(),
                self.browser_control.as_ref().unwrap().clone(),
                &page_id,
                param.screenshot_selector,
            )
            .await?;
        Ok(())
    }
}
