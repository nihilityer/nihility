mod device;
pub mod error;
pub mod func;

use crate::error::*;

use crate::device::register::register_device;
use crate::device::Device;
use axum::extract::ws::WebSocket;
use nihility_module_browser_control::BrowserControl;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tracing::warn;

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
    /// 设备注册超时时间（秒），默认30秒
    #[serde(default = "default_register_timeout")]
    pub register_timeout_secs: u64,
}

pub struct EdgeDeviceControl {
    web_socket_sender: mpsc::UnboundedSender<WebSocket>,
    devices: Arc<RwLock<HashMap<String, Device>>>,
    browser_control: Option<Arc<RwLock<BrowserControl>>>,
    web_socket_receive_task: JoinHandle<Result<()>>,
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

        let (web_socket_sender, mut web_socket_receiver) = mpsc::unbounded_channel::<WebSocket>();

        let web_socket_devices = devices.clone();
        let web_socket_receive_task = tokio::spawn(async move {
            while let Some(web_socket) = web_socket_receiver.recv().await {
                match timeout(
                    Duration::from_secs(config.register_timeout_secs),
                    register_device(web_socket, web_socket_devices.clone()),
                )
                .await
                {
                    Ok(register_result) => register_result?,
                    Err(e) => {
                        warn!("register device timeout: {}", e);
                    }
                }
            }
            Result::<()>::Ok(())
        });

        Ok(EdgeDeviceControl {
            web_socket_sender,
            devices,
            browser_control: None,
            web_socket_receive_task,
        })
    }

    /// 停止接受新的设备连接
    pub fn stop_register_devices(&self) {
        self.web_socket_receive_task.abort();
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

fn default_register_timeout() -> u64 {
    30
}

impl Default for EdgeDeviceControlConfig {
    fn default() -> Self {
        Self {
            register_timeout_secs: default_register_timeout(),
        }
    }
}
