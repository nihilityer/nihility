mod device;
pub mod error;
pub mod func;
mod utils;

use crate::error::*;

use crate::device::Device;
use crate::utils::discovery::start_discovery;
use nihility_module_audio::AudioModule;
use nihility_module_browser_control::BrowserControl;
use nihility_module_model::ModelModule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EdgeDeviceControlConfig {
    pub mdns_service_type: String,
}

pub struct EdgeDeviceControl {
    devices: Arc<RwLock<HashMap<String, Device>>>,
    browser_control: Option<Arc<RwLock<BrowserControl>>>,
    audio_module: Option<Arc<AudioModule>>,
    model_module: Option<Arc<RwLock<ModelModule>>>,
    asr_result_tx: broadcast::Sender<AsrResult>,
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

        // 创建 ASR 结果广播 channel
        let (asr_result_tx, _) = broadcast::channel(32);

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
            audio_module: None,
            model_module: None,
            asr_result_tx,
        })
    }

    /// 设置浏览器控制引用
    pub fn set_browser_control(&mut self, browser: Arc<RwLock<BrowserControl>>) {
        self.browser_control = Some(browser);
    }

    /// 设置音频模块引用
    pub fn set_audio_module(&mut self, audio: Arc<AudioModule>) {
        self.audio_module = Some(audio);
    }

    /// 设置模型模块引用
    pub fn set_model_module(&mut self, model: Arc<RwLock<ModelModule>>) {
        self.model_module = Some(model);
    }

    /// 订阅 ASR 识别结果
    pub fn subscribe_asr_results(&self) -> broadcast::Receiver<AsrResult> {
        self.asr_result_tx.subscribe()
    }

    /// 广播 ASR 识别结果
    pub(crate) fn broadcast_asr_result(&self, result: AsrResult) {
        // 忽略发送失败的接收者
        let _ = self.asr_result_tx.send(result);
    }

    /// 获取音频模块引用
    pub fn audio_module(&self) -> Option<&Arc<AudioModule>> {
        self.audio_module.as_ref()
    }

    /// 获取模型模块引用
    pub fn model_module(&self) -> Option<&Arc<RwLock<ModelModule>>> {
        self.model_module.as_ref()
    }
}

impl Default for EdgeDeviceControlConfig {
    fn default() -> Self {
        Self {
            mdns_service_type: "_edge-device._tcp.local.".to_string(),
        }
    }
}
