use crate::{
    key::KeyEvent,
    screen::{FullScreenData, IncrementalScreenData},
};
use alloc::string::String;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

/// 双向消息枚举
#[derive(Debug, Clone, Archive, RkyvSerialize, RkyvDeserialize)]
pub enum Message {
    // 设备发送的消息
    KeyEvent(KeyEvent),

    // 控制模块发送的消息
    FullScreenUpdate(FullScreenData),
    IncrementalScreenUpdate(IncrementalScreenData),
}

/// 设备信息
#[derive(Debug, Clone, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub screen_width: u16,
    pub screen_height: u16,
    pub screen_refresh_interval: usize,
}
