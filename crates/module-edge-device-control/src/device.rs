use crate::device::connect_ws::connect_ws;
use crate::device::task::message_handler::start_message_handler;
use crate::device::task::screen_refresh::start_screen_refresh;
use crate::error::*;
use nihility_edge_protocol::{DeviceInfo, Message};
use nihility_module_browser_control::BrowserControl;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;

pub mod connect_ws;
mod screen_processor;
mod task;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceStatus {
    Discovered,
    Connected,
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
        }
    }

    pub async fn connect(
        &mut self,
        devices: Arc<RwLock<HashMap<String, Device>>>,
        browser_control: Arc<RwLock<BrowserControl>>,
        page_id: &str,
        screenshot_selector: Option<String>,
    ) -> Result<()> {
        if self.status == DeviceStatus::Connected {
            return Err(EdgeDeviceControlError::DeviceStatus(format!(
                "device {} is already connected",
                self.info.device_id
            )));
        }
        if let Some(addr) = &self.addr {
            let (tx, rx) = connect_ws(*addr).await?;
            self.ws_sender = Some(tx);
            start_message_handler(
                self.info.device_id.clone(),
                devices.clone(),
                browser_control.clone(),
                rx,
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
        )
        .await?;
        self.screen_refresh_task = Some(screen_refresh_task);

        self.status = DeviceStatus::Connected;

        Ok(())
    }
}
