mod epd;
mod epd_trait;

use crate::display::epd::EpdInterface;
use crate::display::epd_trait::EpdDisplay;
use anyhow::Result;
use core::cell::Cell;
use critical_section::Mutex;
use epd::Display;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::peripherals::{GPIO10, GPIO11, GPIO12, GPIO13, GPIO6, GPIO8, GPIO9, SPI3};
use esp_hal::spi::master::Spi;
use esp_hal::spi::{BitOrder, Mode};
use esp_hal::time::Rate;
use log::{error, info};
use nihility_edge_protocol::UpdateRegion;

const WIDTH: u16 = 400;
const HEIGHT: u16 = 300;

static DISPLAY: Mutex<Cell<Option<Display>>> = Mutex::new(Cell::new(None));

pub fn init_display(
    busy: GPIO8<'static>,
    reset: GPIO9<'static>,
    dc: GPIO10<'static>,
    cs: GPIO11<'static>,
    spi3: SPI3<'static>,
    sck: GPIO12<'static>,
    sio: GPIO13<'static>,
) -> Result<()> {
    let busy = Input::new(busy, InputConfig::default());
    let reset = Output::new(reset, Level::Low, OutputConfig::default());
    let dc = Output::new(dc, Level::Low, OutputConfig::default());
    let cs = Output::new(cs, Level::Low, OutputConfig::default());

    let spi = Spi::new(
        spi3,
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
        if let Some(mut display) = DISPLAY.borrow(cs).take() {
            info!("full screen update");
            if let Err(e) = EpdDisplay::init(&mut display) {
                error!("Failed to init display: {:?}", e);
                DISPLAY.borrow(cs).replace(Some(display));
                return Err(anyhow::anyhow!("Display init failed"));
            }
            if let Err(e) = EpdDisplay::full_update(&mut display, data) {
                error!("Failed to update display: {:?}", e);
                DISPLAY.borrow(cs).replace(Some(display));
                return Err(anyhow::anyhow!("Display update failed"));
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
pub fn incremental_screen_update(regions: &[UpdateRegion]) -> Result<()> {
    critical_section::with(|cs| {
        if let Some(mut display) = DISPLAY.borrow(cs).take() {
            info!("Incremental update: {} region(s)", regions.len());
            if let Err(e) = EpdDisplay::init(&mut display) {
                error!("Failed to init display: {:?}", e);
                DISPLAY.borrow(cs).replace(Some(display));
                return Err(anyhow::anyhow!("Display init failed"));
            }

            for (i, region) in regions.iter().enumerate() {
                if let Err(e) = EpdDisplay::partial_update(
                    &mut display,
                    region.x,
                    region.y,
                    region.width,
                    region.height,
                    &region.data,
                ) {
                    error!("Failed to update region {}: {:?}", i, e);
                    DISPLAY.borrow(cs).replace(Some(display));
                    return Err(anyhow::anyhow!("Display partial update failed"));
                }
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
