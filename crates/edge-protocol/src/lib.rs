#![no_std]

extern crate alloc;

pub mod audio;
pub mod key;
pub mod message;
pub mod screen;

pub use audio::AudioData;
pub use key::{KeyCode, KeyEvent};
pub use message::Message;
pub use screen::{FullScreenData, IncrementalScreenData, UpdateRegion};
