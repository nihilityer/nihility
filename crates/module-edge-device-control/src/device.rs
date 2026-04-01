use crate::device::connect_ws::connect_ws;
use crate::device::task::message_handler::start_message_handler;
use crate::device::task::screen_refresh::start_screen_refresh;
use crate::error::*;
use crate::AsrResult;
use nihility_edge_protocol::Message;
use nihility_module_browser_control::BrowserControl;
use nihility_module_model::ModelModule;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::info;

pub mod connect_ws;
mod screen_processor;
mod task;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceStatus {
    Discovered,
    Connected,
}

/// 设备信息
#[derive(Debug, Clone, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub screen_width: u16,
    pub screen_height: u16,
    pub screen_refresh_interval: usize,
    pub screen_config: ScreenConfig,
}

/// 屏幕旋转角度
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Archive, RkyvSerialize, RkyvDeserialize)]
pub enum ScreenRotation {
    /// 不旋转（0度）
    #[default]
    Rotate0,
    /// 顺时针旋转90度
    Rotate90,
    /// 旋转180度
    Rotate180,
    /// 顺时针旋转270度（逆时针90度）
    Rotate270,
}

/// 屏幕配置
#[derive(Debug, Clone, Copy, PartialEq, Default, Eq, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct ScreenConfig {
    /// 旋转角度
    pub rotation: ScreenRotation,
    /// 水平镜像
    pub mirror_horizontal: bool,
    /// 垂直镜像
    pub mirror_vertical: bool,
}

#[derive(Debug)]
pub struct Device {
    pub info: DeviceInfo,
    pub addr: Option<SocketAddr>,
    pub status: DeviceStatus,
    pub ws_sender: Option<mpsc::UnboundedSender<Message>>,
    pub page_id: Option<String>,
    pub screen_refresh_task: Option<JoinHandle<Result<()>>>,
    pub screen_refresh_task_switch: bool,
    pub cancellation_token: CancellationToken,
}

impl Device {
    pub fn new(info: DeviceInfo) -> Self {
        Self {
            info,
            addr: None,
            status: DeviceStatus::Discovered,
            ws_sender: None,
            page_id: None,
            screen_refresh_task: None,
            screen_refresh_task_switch: true,
            cancellation_token: CancellationToken::new(),
        }
    }

    pub async fn connect(
        &mut self,
        devices: Arc<RwLock<HashMap<String, Device>>>,
        browser_control: Arc<RwLock<BrowserControl>>,
        model_module: Arc<RwLock<ModelModule>>,
        asr_result_tx: Arc<broadcast::Sender<AsrResult>>,
        page_id: &str,
        screenshot_selector: Option<String>,
    ) -> Result<()> {
        if self.status == DeviceStatus::Connected {
            return Err(EdgeDeviceControlError::DeviceStatus(format!(
                "device {} is already connected",
                self.info.device_id
            )));
        }

        // 创建新的取消 token
        let cancellation_token = CancellationToken::new();
        self.cancellation_token = cancellation_token.clone();

        if let Some(addr) = &self.addr {
            let (tx, rx) = connect_ws(*addr, cancellation_token.clone()).await?;
            self.ws_sender = Some(tx);

            // 启动断开监听任务
            let device_id = self.info.device_id.clone();
            let devices_clone = devices.clone();
            let cancellation_token_for_listener = cancellation_token.clone();

            tokio::spawn(async move {
                cancellation_token_for_listener.cancelled().await;
                // 连接断开，更新设备状态为 Discovered（可重连）
                let mut devices_guard = devices_clone.write().await;
                if let Some(device) = devices_guard.get_mut(&device_id) {
                    info!(
                        "Device {} disconnected, setting status to Discovered",
                        device_id
                    );
                    device.status = DeviceStatus::Discovered;
                    device.ws_sender = None;
                }
            });

            start_message_handler(
                self.info.device_id.clone(),
                devices.clone(),
                browser_control.clone(),
                model_module,
                asr_result_tx,
                rx,
                cancellation_token.clone(),
            )
            .await?;
        } else {
            // TODO 暂时只支持服务端主动连接设备
            return Err(EdgeDeviceControlError::DeviceStatus(format!(
                "device {} is not connected",
                self.info.device_id
            )));
        }
        let screen_refresh_task = start_screen_refresh(
            self.info.clone(),
            devices.clone(),
            browser_control.clone(),
            page_id,
            screenshot_selector.clone(),
            cancellation_token,
        )
        .await?;
        self.screen_refresh_task = Some(screen_refresh_task);

        self.status = DeviceStatus::Connected;

        Ok(())
    }
}
