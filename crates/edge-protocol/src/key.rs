use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEvent {
    pub key_code: KeyCode,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyCode {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Back,
    Custom(u8),
}

impl KeyCode {
    /// 转换为浏览器模块的按键字符串
    pub fn to_browser_key(&self) -> String {
        match self {
            KeyCode::Up => "ArrowUp".to_string(),
            KeyCode::Down => "ArrowDown".to_string(),
            KeyCode::Left => "ArrowLeft".to_string(),
            KeyCode::Right => "ArrowRight".to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Back => "Backspace".to_string(),
            KeyCode::Custom(code) => format!("Custom{}", code),
        }
    }
}
