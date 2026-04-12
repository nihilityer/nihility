use crate::device::task::key_handle::start_key_handle;
use crate::device::task::screen_refresh::start_screen_refresh;
use crate::error::*;
use nihility_edge_protocol::{DeviceInfo, KeyCode, Message};
use nihility_module_browser_control::BrowserControl;
use std::collections::HashMap;
use std::sync::Arc;
pub(crate) use task::message_handle::start_message_handle;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub mod register;
mod screen_processor;
mod task;

#[derive(Debug)]
pub struct Device {
    pub info: DeviceInfo,
    pub page_id: Option<Uuid>,
    pub key_sender: Option<mpsc::UnboundedSender<KeyCode>>,
    pub ws_sender: Option<mpsc::UnboundedSender<Message>>,
    pub key_handle_task: Option<JoinHandle<Result<()>>>,
    pub screen_refresh_task: Option<JoinHandle<Result<()>>>,
    pub cancellation_token: CancellationToken,
    pub audio_vad_task: Option<
        JoinHandle<core::result::Result<(), nihility_util_vad::error::VoiceActivityDetectionError>>,
    >,
    pub audio_handle_task: Option<JoinHandle<Result<()>>>,
}

impl Device {
    pub fn new(info: DeviceInfo) -> Self {
        Self {
            info,
            page_id: None,
            key_sender: None,
            ws_sender: None,
            key_handle_task: None,
            screen_refresh_task: None,
            cancellation_token: CancellationToken::new(),
            audio_vad_task: None,
            audio_handle_task: None,
        }
    }

    pub async fn start_key_handle(
        &mut self,
        browser_control: Arc<RwLock<BrowserControl>>,
        page_id: Uuid,
    ) -> Result<()> {
        let (key_sender, key_receiver) = mpsc::unbounded_channel();
        self.key_sender = Some(key_sender);
        self.key_handle_task =
            Some(start_key_handle(page_id.to_string(), key_receiver, browser_control).await?);
        Ok(())
    }

    pub async fn start_screen_push(
        &mut self,
        devices: Arc<RwLock<HashMap<String, Device>>>,
        browser_control: Arc<RwLock<BrowserControl>>,
        page_id: Uuid,
        screenshot_selector: Option<String>,
    ) -> Result<()> {
        self.screen_refresh_task = Some(
            start_screen_refresh(
                self.info.clone(),
                devices.clone(),
                browser_control.clone(),
                page_id,
                screenshot_selector.clone(),
                self.cancellation_token.clone(),
            )
            .await?,
        );

        Ok(())
    }
}
