use alloc::string::String;
use serde::{Deserialize, Serialize};

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub screen_width: u16,
    pub screen_height: u16,
    pub screen_refresh_interval: usize,
    pub screen_config: ScreenConfig,
}

/// 屏幕旋转角度
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Default, Eq, Serialize, Deserialize)]
pub struct ScreenConfig {
    /// 旋转角度
    pub rotation: ScreenRotation,
    /// 水平镜像
    pub mirror_horizontal: bool,
    /// 垂直镜像
    pub mirror_vertical: bool,
}
