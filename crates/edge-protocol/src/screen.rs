use alloc::vec::Vec;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

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

/// 完整屏幕数据（初次连接或全量刷新）
#[derive(Debug, Clone, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct FullScreenData {
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
    pub timestamp: u64,
}

/// 增量屏幕更新（只传输变化区域）
#[derive(Debug, Clone, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct IncrementalScreenData {
    pub regions: Vec<UpdateRegion>,
    pub timestamp: u64,
}

/// 矩形更新区域
#[derive(Debug, Clone, Archive, RkyvSerialize, RkyvDeserialize)]
pub struct UpdateRegion {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub data: Vec<u8>,
}
