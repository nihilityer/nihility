mod device;
pub mod error;
pub mod func;
mod utils;

use crate::error::*;

use crate::device::Device;
use crate::utils::discovery::start_discovery;
use axum::extract::ws::WebSocket;
use nihility_module_browser_control::BrowserControl;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// 语音识别结果
#[derive(Debug, Clone)]
pub struct AsrResult {
    /// 设备ID
    pub device_id: String,
    /// 识别文本
    pub text: String,
    /// 时间戳（毫秒）
    pub timestamp: u64,
}

/// 边缘设备控制模块配置
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EdgeDeviceControlConfig {
    /// mDNS 服务类型，用于设备发现
    /// 格式为 "_service._protocol.local."
    /// 例如："_edge-device._tcp.local."
    pub mdns_service_type: String,
}

pub struct EdgeDeviceControl {
    web_socket_sender: mpsc::UnboundedSender<WebSocket>,
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

        let (web_socket_sender, _web_socket_receiver) = mpsc::unbounded_channel();
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
            web_socket_sender,
            devices,
            browser_control: None,
        })
    }

    /// 设置浏览器控制引用
    pub fn set_browser_control(&mut self, browser: Arc<RwLock<BrowserControl>>) {
        self.browser_control = Some(browser);
    }

    /// 获取WebSocket的发送者，用于传递设备的WebSocket流到设备控制模块
    pub fn get_web_socket_sender(&self) -> mpsc::UnboundedSender<WebSocket> {
        self.web_socket_sender.clone()
    }
}

impl Default for EdgeDeviceControlConfig {
    fn default() -> Self {
        Self {
            mdns_service_type: "_edge-device._tcp.local.".to_string(),
        }
    }
}
