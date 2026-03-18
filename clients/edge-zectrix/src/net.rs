use alloc::string::{String, ToString};
use anyhow::anyhow;
use anyhow::Result;
use esp_hal::efuse::Efuse;

pub mod mdns;
pub mod wifi;

pub(crate) const MAX_RETRY_COUNT: usize = 5;
pub(crate) const SERVICE_PORT: u16 = 5000;

pub fn get_device_id() -> Result<String> {
    let mut buf = [0u8; 8];
    let mac = Efuse::read_base_mac_address();
    hex::encode_to_slice(&mac[2..], &mut buf)
        .map_err(|_| anyhow!("Failed to parse MAC address"))?;
    Ok(core::str::from_utf8(&buf)?.to_string())
}
