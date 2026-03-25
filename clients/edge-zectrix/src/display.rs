mod epd;

use crate::display::epd::EpdInterface;
use anyhow::Result;
use core::cell::Cell;
use critical_section::Mutex;
use epd::ssd1683::{DeepSleepMode, Display};
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::peripherals::{GPIO10, GPIO11, GPIO12, GPIO13, GPIO8, GPIO9, SPI3};
use esp_hal::spi::master::Spi;
use esp_hal::spi::{BitOrder, Mode};
use esp_hal::time::Rate;
use log::{error, info};
use nihility_edge_protocol::UpdateRegion;

const WIDTH: u16 = 400;
const HEIGHT: u16 = 300;
const MAX_FAST_UPDATES: usize = 10;

static DISPLAY: Mutex<Cell<Option<Display>>> = Mutex::new(Cell::new(None));

static UPDATE_COUNT: Mutex<Cell<usize>> = Mutex::new(Cell::new(0));

pub fn init_display(
    busy: GPIO8<'static>,
    reset: GPIO9<'static>,
    dc: GPIO10<'static>,
    cs: GPIO11<'static>,
    spi2: SPI3<'static>,
    sck: GPIO12<'static>,
    sio: GPIO13<'static>,
) -> Result<()> {
    let busy = Input::new(busy, InputConfig::default());
    let reset = Output::new(reset, Level::Low, OutputConfig::default());
    let dc = Output::new(dc, Level::Low, OutputConfig::default());
    let cs = Output::new(cs, Level::Low, OutputConfig::default());

    let spi = Spi::new(
        spi2,
        esp_hal::spi::master::Config::default()
            .with_mode(Mode::_0)
            .with_frequency(Rate::from_mhz(10))
            .with_write_bit_order(BitOrder::MsbFirst),
    )?
    .with_sck(sck)
    .with_cs(cs)
    .with_sio0(sio);

    let interface = EpdInterface::new(spi, busy, reset, dc);
    let display = Display::new(interface, Delay::new(), WIDTH as usize, HEIGHT);

    critical_section::with(|cs| {
        DISPLAY.borrow(cs).replace(Some(display));
    });
    Ok(())
}

/// 全量屏幕更新
pub fn full_screen_update(data: &[u8]) -> Result<()> {
    critical_section::with(|cs| {
        let mut count = UPDATE_COUNT.borrow(cs).take();
        if let Some(mut display) = DISPLAY.borrow(cs).take() {
            if count == 0 || count > MAX_FAST_UPDATES {
                info!("normal full screen update");
                // 初始化显示屏
                if let Err(e) = display.normal_init() {
                    error!("Failed to normal init display: {:?}", e);
                    DISPLAY.borrow(cs).replace(Some(display));
                    return Err(anyhow::anyhow!("Display init failed"));
                }
            } else {
                info!("fast full screen update");
                // 快速刷新初始化
                if let Err(e) = display.fast_init(false) {
                    error!("Failed to fast init display: {:?}", e);
                    DISPLAY.borrow(cs).replace(Some(display));
                    return Err(anyhow::anyhow!("Display init failed"));
                }
            }

            // 写入完整屏幕数据
            if let Err(e) = display.write_all(data) {
                error!("Failed to write display data: {:?}", e);
                DISPLAY.borrow(cs).replace(Some(display));
                return Err(anyhow::anyhow!("Display write failed"));
            }

            if count == 0 || count > MAX_FAST_UPDATES {
                count = 1;
                // 执行正常更新
                if let Err(e) = display.normal_update() {
                    error!("Failed to normal update display: {:?}", e);
                    DISPLAY.borrow(cs).replace(Some(display));
                    return Err(anyhow::anyhow!("Display update failed"));
                }
            } else {
                count += 1;
                // 快速刷新
                if let Err(e) = display.fast_update() {
                    error!("Failed to fast update display: {:?}", e);
                    DISPLAY.borrow(cs).replace(Some(display));
                    return Err(anyhow::anyhow!("Display update failed"));
                }
            }
            UPDATE_COUNT.borrow(cs).set(count);

            // 进入深度睡眠以节省电量，保留 RAM 内容
            if let Err(e) = display.deep_sleep(DeepSleepMode::PreserveRAM) {
                error!("Failed to enter deep sleep: {:?}", e);
                DISPLAY.borrow(cs).replace(Some(display));
                return Err(anyhow::anyhow!("Display deep sleep failed"));
            }

            DISPLAY.borrow(cs).replace(Some(display));
            Ok(())
        } else {
            error!("Display not initialized");
            Err(anyhow::anyhow!("Display not initialized"))
        }
    })
}

/// 增量屏幕更新
/// 基于 SSD1683 示例的 part_write -> part_update 流程
pub fn incremental_screen_update(regions: &[UpdateRegion]) -> Result<()> {
    critical_section::with(|cs| {
        if let Some(mut display) = DISPLAY.borrow(cs).take() {
            info!("Incremental update: {} region(s)", regions.len());

            // 对每个区域执行部分写入
            for (i, region) in regions.iter().enumerate() {
                // 转换Y坐标
                let ssd1683_y = HEIGHT - region.y - region.height;

                if let Err(e) = display.part_write(
                    region.x,
                    ssd1683_y,
                    region.width,
                    region.height,
                    &region.data,
                ) {
                    error!("Failed to write region {}: {:?}", i, e);
                    DISPLAY.borrow(cs).replace(Some(display));
                    return Err(anyhow::anyhow!("Display part write failed"));
                }
            }

            // 执行部分更新
            if let Err(e) = display.part_update() {
                error!("Failed to update display: {:?}", e);
                DISPLAY.borrow(cs).replace(Some(display));
                return Err(anyhow::anyhow!("Display part update failed"));
            }

            DISPLAY.borrow(cs).replace(Some(display));
            Ok(())
        } else {
            error!("Display not initialized");
            Err(anyhow::anyhow!("Display not initialized"))
        }
    })
}

/// 初始化显示屏并清空屏幕（显示全白）
pub fn init_and_clear_screen() -> Result<()> {
    const BUFFER_SIZE: usize = (WIDTH as usize * HEIGHT as usize) / 8;
    let buffer = [0xFF_u8; BUFFER_SIZE]; // 全白

    full_screen_update(&buffer)?;
    Ok(())
}
