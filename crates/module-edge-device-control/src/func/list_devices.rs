use crate::EdgeDeviceControl;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 列出所有设备Id
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ListDevicesParam {}

impl EdgeDeviceControl {
    /// 列出所有设备
    pub async fn list_devices(&self) -> Vec<String> {
        let devices = self.devices.read().await;
        devices.keys().cloned().collect()
    }
}
