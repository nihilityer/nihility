use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

/// 完整屏幕数据（初次连接或全量刷新）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullScreenData {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
    pub timestamp: u64,
}

/// 增量屏幕更新（只传输变化区域）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalScreenData {
    pub regions: Vec<UpdateRegion>,
    pub timestamp: u64,
}

/// 矩形更新区域
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRegion {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}
