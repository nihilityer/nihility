use crate::error::*;
use crate::EdgeDeviceControl;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConnectDeviceParam {
    pub device_id: String,
    pub mapping_url: String,
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

        device
            .connect(
                self.devices.clone(),
                self.browser_control.as_ref().unwrap().clone(),
                &param.mapping_url,
                param.screenshot_selector,
            )
            .await?;
        Ok(())
    }
}
