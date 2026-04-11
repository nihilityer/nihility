#![no_std]
#![no_main]
#![feature(allocator_api)]

extern crate alloc;

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use nihility_edge_protocol::Message;

pub mod audio;
pub mod display;
pub mod input;
pub mod net;
pub mod storage;
pub mod work;

static FROM_SERVER_CHANNEL: Channel<CriticalSectionRawMutex, Message, 8> = Channel::new();
static TO_SERVER_CHANNEL: Channel<CriticalSectionRawMutex, Message, 8> = Channel::new();
