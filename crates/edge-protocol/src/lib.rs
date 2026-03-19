#![no_std]

extern crate alloc;

pub mod key;
pub mod message;
pub mod screen;

pub use key::{KeyCode, KeyEvent};
pub use message::{DeviceInfo, Message};
pub use screen::{FullScreenData, IncrementalScreenData, ScreenConfig, ScreenRotation, UpdateRegion};
