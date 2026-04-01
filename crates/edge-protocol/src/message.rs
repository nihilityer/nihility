use crate::{
    audio::AudioData,
    key::KeyEvent,
    screen::{FullScreenData, IncrementalScreenData},
};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

/// 双向消息枚举
#[derive(Debug, Clone, Archive, RkyvSerialize, RkyvDeserialize)]
pub enum Message {
    // 设备发送的消息
    KeyEvent(KeyEvent),
    AudioData(AudioData),

    // 控制模块发送的消息
    FullScreenUpdate(FullScreenData),
    IncrementalScreenUpdate(IncrementalScreenData),
}
