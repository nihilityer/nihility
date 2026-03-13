use crate::{
    key::KeyEvent,
    screen::{FullScreenData, IncrementalScreenData},
};
use serde::{Deserialize, Serialize};

/// 双向消息枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    // 设备发送的消息
    KeyEvent(KeyEvent),

    // 控制模块发送的消息
    FullScreenUpdate(FullScreenData),
    IncrementalScreenUpdate(IncrementalScreenData),
}

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub screen_width: u16,
    pub screen_height: u16,
    pub screen_refresh_interval: usize,
}
