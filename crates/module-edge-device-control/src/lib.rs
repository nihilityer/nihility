mod device;
pub mod error;
pub mod func;

use crate::error::*;

use crate::device::register::register_device;
use crate::device::Device;
use axum::extract::ws::WebSocket;
use nihility_module_browser_control::BrowserControl;
use nihility_module_model::Model;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tracing::{error, info, warn};

/// 自动连接设备配置
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AutoConnectDevice {
    /// 设备Id
    pub device_id: String,
    /// 需要显示在设备屏幕上的网页Url
    pub mapping_url: String,
    /// 屏幕映射网页中哪个元素
    pub screenshot_selector: Option<String>,
}

/// 边缘设备控制模块配置
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EdgeDeviceControlConfig {
    /// 设备注册超时时间（秒），默认30秒
    #[serde(default = "default_register_timeout")]
    pub register_timeout_secs: usize,
    /// 自动连接设备配置列表
    #[serde(default)]
    pub auto_connect: Vec<AutoConnectDevice>,
}

pub struct EdgeDeviceControl {
    web_socket_sender: Option<mpsc::UnboundedSender<WebSocket>>,
    devices: Arc<RwLock<HashMap<String, Device>>>,
    browser_control: Option<Arc<RwLock<BrowserControl>>>,
    model: Option<Arc<RwLock<Model>>>,
    web_socket_receive_task: Option<JoinHandle<Result<()>>>,
    register_timeout_secs: usize,
    auto_connect: Arc<HashMap<String, (String, Option<String>)>>,
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
        let auto_connect: HashMap<String, (String, Option<String>)> = config
            .auto_connect
            .iter()
            .map(|ac| {
                (
                    ac.device_id.clone(),
                    (ac.mapping_url.clone(), ac.screenshot_selector.clone()),
                )
            })
            .collect();
        let module = EdgeDeviceControl {
            web_socket_sender: None,
            devices,
            browser_control: None,
            model: None,
            web_socket_receive_task: None,
            register_timeout_secs: config.register_timeout_secs,
            auto_connect: Arc::new(auto_connect),
        };
        Ok(module)
    }

    pub async fn start_register_device(&mut self) -> Result<()> {
        if self.model.is_none() || self.browser_control.is_none() {
            return Err(EdgeDeviceControlError::DeviceStatus(
                "Module model and browser_control is required".to_string(),
            ));
        }
        let (web_socket_sender, mut web_socket_receiver) = mpsc::unbounded_channel::<WebSocket>();

        let web_socket_devices = self.devices.clone();
        let register_timeout_secs = self.register_timeout_secs;
        let model = self.model.as_ref().unwrap().clone();
        let browser_control = self.browser_control.as_ref().unwrap().clone();
        let auto_connect = self.auto_connect.clone();
        let web_socket_receive_task = tokio::spawn(async move {
            info!("Starting web socket receiver");
            while let Some(web_socket) = web_socket_receiver.recv().await {
                match timeout(
                    Duration::from_secs(register_timeout_secs as u64),
                    register_device(
                        web_socket,
                        model.clone(),
                        web_socket_devices.clone(),
                        browser_control.clone(),
                        auto_connect.clone(),
                    ),
                )
                .await
                {
                    Ok(register_result) => {
                        if let Err(e) = register_result {
                            error!("Failed to register: {:?}", e);
                        }
                    }
                    Err(e) => {
                        warn!("register device timeout: {}", e);
                    }
                }
            }
            Result::<()>::Ok(())
        });
        self.web_socket_sender = Some(web_socket_sender);
        self.web_socket_receive_task = Some(web_socket_receive_task);
        Ok(())
    }

    /// 停止接受新的设备连接
    pub fn stop_register_devices(&mut self) -> Result<()> {
        info!("Stopping registration device");
        if let Some(task) = &self.web_socket_receive_task {
            task.abort();
            self.web_socket_receive_task = None;
            self.web_socket_sender = None;
        }
        Ok(())
    }

    /// 设置浏览器控制引用
    pub fn set_browser_control(&mut self, browser: Arc<RwLock<BrowserControl>>) {
        self.browser_control = Some(browser);
    }

    /// 设置模型模块引用
    pub async fn set_model(&mut self, model: Arc<RwLock<Model>>) -> Result<()> {
        self.model = Some(model);
        Ok(())
    }

    /// 获取WebSocket的发送者，用于传递设备的WebSocket流到设备控制模块
    pub fn get_web_socket_sender(&self) -> Result<mpsc::UnboundedSender<WebSocket>> {
        Ok(self
            .web_socket_sender
            .as_ref()
            .ok_or(EdgeDeviceControlError::ModuleStatus(
                "WebSocket sender not init".to_string(),
            ))?
            .clone())
    }
}

fn default_register_timeout() -> usize {
    30
}

impl Default for EdgeDeviceControlConfig {
    fn default() -> Self {
        Self {
            register_timeout_secs: default_register_timeout(),
            auto_connect: Vec::new(),
        }
    }
}

pub async fn monitor_task(module: Arc<RwLock<EdgeDeviceControl>>) {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
        let mut module = module.write().await;
        if let Some(task) = module.web_socket_receive_task.as_ref()
            && task.is_finished()
            && let Some(task) = module.web_socket_receive_task.take()
        {
            match task.await {
                Ok(Ok(())) => info!("WebSocket receive task finished"),
                Ok(Err(e)) => {
                    error!("WebSocket receive task failed: {}", e);
                }
                Err(join_err) => {
                    error!("WebSocket receive task join failed: {}", join_err);
                }
            }
        }

        let mut devices = module.devices.write().await;

        for device in devices.values_mut() {
            if let Some(task) = device.audio_vad_task.as_ref()
                && task.is_finished()
                && let Some(task) = device.audio_vad_task.take()
            {
                match task.await {
                    Ok(Ok(())) => info!("device {} audio vad task finished", device.info.device_id),
                    Ok(Err(e)) => {
                        error!(
                            "device {} audio vad task failed: {}",
                            device.info.device_id, e
                        );
                    }
                    Err(join_err) => {
                        error!(
                            "device {} audio vad task join failed: {}",
                            device.info.device_id, join_err
                        );
                    }
                }
            }
            if let Some(task) = device.key_handle_task.as_ref()
                && task.is_finished()
                && let Some(task) = device.key_handle_task.take()
            {
                match task.await {
                    Ok(Ok(())) => {
                        info!("device {} key handle task finished", device.info.device_id)
                    }
                    Ok(Err(e)) => {
                        error!(
                            "device {} key handle task failed: {}",
                            device.info.device_id, e
                        );
                    }
                    Err(join_err) => {
                        error!(
                            "device {} key handle task join failed: {}",
                            device.info.device_id, join_err
                        );
                    }
                }
            }
            if let Some(task) = device.audio_handle_task.as_ref()
                && task.is_finished()
                && let Some(task) = device.audio_handle_task.take()
            {
                match task.await {
                    Ok(Ok(())) => info!(
                        "device {} audio handle task finished",
                        device.info.device_id
                    ),
                    Ok(Err(e)) => {
                        error!(
                            "device {} audio handle task failed: {}",
                            device.info.device_id, e
                        );
                    }
                    Err(join_err) => {
                        error!(
                            "device {} audio handle task join failed: {}",
                            device.info.device_id, join_err
                        );
                    }
                }
            }
            if let Some(task) = device.screen_refresh_task.as_ref()
                && task.is_finished()
                && let Some(task) = device.screen_refresh_task.take()
            {
                match task.await {
                    Ok(Ok(())) => info!(
                        "device {} screen refresh task finished",
                        device.info.device_id
                    ),
                    Ok(Err(e)) => {
                        error!(
                            "device {} screen refresh task failed: {}",
                            device.info.device_id, e
                        );
                    }
                    Err(join_err) => {
                        error!(
                            "device {} screen refresh task join failed: {}",
                            device.info.device_id, join_err
                        );
                    }
                }
            }
        }
    }
}
