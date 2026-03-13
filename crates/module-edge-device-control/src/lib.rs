mod device;
pub mod error;
pub mod func;
mod utils;

use crate::error::*;

use crate::device::Device;
use crate::utils::discovery::start_discovery;
use nihility_module_browser_control::BrowserControl;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgeDeviceControlConfig {
    pub mdns_service_type: String,
}

pub struct EdgeDeviceControl {
    devices: Arc<RwLock<HashMap<String, Device>>>,
    browser_control: Option<Arc<RwLock<BrowserControl>>>,
}

impl EdgeDeviceControl {
    pub async fn init_from_file_config() -> Result<Self> {
        Self::init(nihility_config::get_config::<EdgeDeviceControlConfig>(
            env!("CARGO_PKG_NAME"),
        )?)
        .await
    }

    pub async fn init(config: EdgeDeviceControlConfig) -> Result<Self> {
        let devices = Arc::new(RwLock::new(HashMap::new()));

        // 启动 mDNS 发现
        let (tx, mut rx) = mpsc::unbounded_channel();
        start_discovery(&config.mdns_service_type, tx)?;

        // 监听发现事件
        let devices_clone = devices.clone();
        tokio::spawn(async move {
            while let Some((addr, device_info)) = rx.recv().await {
                let mut devices = devices_clone.write().await;
                let device = devices
                    .entry(device_info.device_id.clone())
                    .or_insert_with(|| Device::new(device_info));
                device.addr = Some(addr);
            }
            Result::Ok(())
        });

        Ok(EdgeDeviceControl {
            devices,
            browser_control: None,
        })
    }

    /// 设置浏览器控制引用
    pub fn set_browser_control(&mut self, browser: Arc<RwLock<BrowserControl>>) {
        self.browser_control = Some(browser);
    }
}

impl Default for EdgeDeviceControlConfig {
    fn default() -> Self {
        Self {
            mdns_service_type: "_edge-device._tcp.local.".to_string(),
        }
    }
}
