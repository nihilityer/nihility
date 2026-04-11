use alloc::string::{String, ToString};
use alloc::vec::Vec;
use anyhow::{anyhow, Result};
use core::cell::Cell;
use critical_section::Mutex;
use embedded_storage::nor_flash::{NorFlash, ReadNorFlash};
use esp_hal::peripherals::FLASH;
use esp_storage::FlashStorage;
use log::{error, info};
use postcard::{from_bytes, to_allocvec};
use serde::{Deserialize, Serialize};

static STORAGE: Mutex<Cell<Option<FlashStorage>>> = Mutex::new(Cell::new(None));

/// WiFi 凭证结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WifiCredentials {
    pub ssid: String,
    pub password: String,
}

/// 服务器配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

/// 完整配置结构（存储 WiFi 凭证和服务器配置）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    pub wifi: WifiCredentials,
    pub server: ServerConfig,
}

/// 存储偏移量（从NVS分区后的自定义存储区域开始）
pub const CREDENTIALS_OFFSET: u32 = 0x9000;
/// Flash 擦除块大小 (4KB)
pub const ERASE_SIZE: u32 = 4096;
/// 最大 SSID 长度
pub const MAX_SSID_LEN: usize = 32;
/// 最大密码长度
pub const MAX_PASSWORD_LEN: usize = 64;
/// 魔术字节用于验证数据有效性
const MAGIC_BYTES: [u8; 4] = [0xCA, 0xFE, 0xBA, 0xBE];

/// 初始化存储
pub fn init_storage(flash: FLASH<'static>) -> Result<()> {
    critical_section::with(|cs| {
        STORAGE.borrow(cs).replace(Some(FlashStorage::new(flash)));
    });
    Ok(())
}

/// 保存完整配置到 Flash
pub fn save_config(config: &DeviceConfig) -> Result<()> {
    // 验证输入
    if config.wifi.ssid.len() > MAX_SSID_LEN {
        return Err(anyhow!("ssid too long"));
    }
    if config.wifi.password.len() > MAX_PASSWORD_LEN {
        return Err(anyhow!("password too long"));
    }

    let serialized = to_allocvec(config)?;
    let data_len = serialized.len();

    // 准备写入数据：魔术字节 + 长度 + 序列化数据
    let mut write_buffer = Vec::new_in(esp_alloc::ExternalMemory);
    write_buffer.extend_from_slice(&MAGIC_BYTES);
    write_buffer.extend_from_slice(&(data_len as u32).to_le_bytes());
    write_buffer.extend_from_slice(serialized.as_slice());

    critical_section::with(|cs| {
        let mut flash_storage = STORAGE
            .borrow(cs)
            .replace(None)
            .expect("flash storage not init");
        // 擦除 Flash 扇区
        flash_storage
            .erase(CREDENTIALS_OFFSET, CREDENTIALS_OFFSET + ERASE_SIZE)
            .expect("Failed to erase credentials");

        // 写入数据
        flash_storage
            .write(CREDENTIALS_OFFSET, &write_buffer)
            .expect("Failed to write credentials");
        STORAGE.borrow(cs).replace(Some(flash_storage));
    });

    info!(
        "Device config saved to flash at offset 0x{:X}",
        CREDENTIALS_OFFSET
    );
    Ok(())
}

/// 从 Flash 加载完整配置
///
/// 如果没有保存的配置或数据无效，返回 None
pub fn load_config() -> Result<Option<DeviceConfig>> {
    // 读取魔术字节
    let mut magic = Vec::with_capacity_in(4, esp_alloc::ExternalMemory);
    magic.resize(4, 0);
    critical_section::with(|cs| {
        let mut flash_storage = STORAGE
            .borrow(cs)
            .replace(None)
            .expect("flash storage not init");
        flash_storage
            .read(CREDENTIALS_OFFSET, &mut magic)
            .expect("Failed to read credentials");
        STORAGE.borrow(cs).replace(Some(flash_storage));
    });

    // 验证魔术字节
    if magic != MAGIC_BYTES {
        info!("No valid config found in flash (invalid magic bytes)");
        return Ok(None);
    }

    // 读取数据长度
    let mut len_bytes = [0u8; 4];
    critical_section::with(|cs| {
        let mut flash_storage = STORAGE
            .borrow(cs)
            .replace(None)
            .expect("flash storage not init");
        flash_storage
            .read(CREDENTIALS_OFFSET + 4, &mut len_bytes)
            .expect("Failed to read credentials");
        STORAGE.borrow(cs).replace(Some(flash_storage));
    });
    let data_len = u32::from_le_bytes(len_bytes) as usize;

    // 验证长度
    if data_len == 0 {
        error!("Invalid data length in flash: {}", data_len);
        return Ok(None);
    }

    // 读取序列化数据
    let mut buffer = Vec::new_in(esp_alloc::ExternalMemory);
    buffer.resize(data_len, 0);
    critical_section::with(|cs| {
        let mut flash_storage = STORAGE
            .borrow(cs)
            .replace(None)
            .expect("flash storage not init");
        flash_storage
            .read(CREDENTIALS_OFFSET + 8, &mut buffer[..data_len])
            .expect("Failed to read credentials");
        STORAGE.borrow(cs).replace(Some(flash_storage));
    });

    // 反序列化
    match from_bytes::<DeviceConfig>(&buffer[..data_len]) {
        Ok(mut config) => {
            info!(
                "Loaded config from flash: SSID={}, Server={}:{}",
                config.wifi.ssid, config.server.host, config.server.port
            );
            Ok(Some(config))
        }
        Err(e) => {
            error!("Failed to deserialize config: {:?}", e);
            Ok(None)
        }
    }
}

/// 清空保存的 WiFi 凭证
pub fn clear_credentials() -> Result<()> {
    critical_section::with(|cs| {
        let mut flash_storage = STORAGE
            .borrow(cs)
            .replace(None)
            .expect("flash storage not init");
        // 擦除 Flash 扇区
        flash_storage
            .erase(CREDENTIALS_OFFSET, CREDENTIALS_OFFSET + ERASE_SIZE)
            .expect("Failed to erase credentials");
        STORAGE.borrow(cs).replace(Some(flash_storage));
    });
    info!("Cleared WiFi credentials from flash");
    Ok(())
}
