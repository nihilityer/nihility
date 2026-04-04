use crate::device_info::DeviceInfo;
use crate::{
    audio::AudioData,
    key::KeyEvent,
    screen::{FullScreenData, IncrementalScreenData},
};
use serde::{Deserialize, Serialize};

/// 双向消息枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    // 设备发送的消息
    DeviceInfo(DeviceInfo),
    KeyEvent(KeyEvent),
    AudioData(AudioData),

    // 控制模块发送的消息
    FullScreenUpdate(FullScreenData),
    IncrementalScreenUpdate(IncrementalScreenData),
}
