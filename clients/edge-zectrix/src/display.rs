mod epd;
mod epd_trait;

use crate::display::epd::EpdInterface;
use crate::display::epd_trait::EpdDisplay;
use alloc::vec::Vec;
use anyhow::Result;
use core::cell::{Cell, RefCell};
use critical_section::Mutex;
use epd::Display;
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
const FRAME_SIZE: usize = (WIDTH as usize * HEIGHT as usize) / 8;

/// 帧缓存，用于局部刷新时保存前一帧数据
static FRAME_BUFFER: Mutex<RefCell<[u8; FRAME_SIZE]>> =
    Mutex::new(RefCell::new([0xFF; FRAME_SIZE]));

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

            // 更新帧缓存
            FRAME_BUFFER.borrow(cs).borrow_mut().copy_from_slice(data);

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

            let mut frame = FRAME_BUFFER.borrow(cs).borrow_mut();
            let width_bytes = WIDTH as usize / 8;

            for (i, region) in regions.iter().enumerate() {
                // 从帧缓存中提取局部区域的前一帧数据
                let w_bytes = ((region.width + 7) / 8) as usize;
                let h = region.height as usize;
                let expected_len = w_bytes * h;

                // 栈上分配临时数组
                let mut prev_data = Vec::new_in(esp_alloc::ExternalMemory);
                prev_data.resize(expected_len, 0);

                // 计算区域在帧缓存中的偏移
                let start_byte = (region.y as usize * width_bytes) + (region.x as usize / 8);

                for row in 0..h {
                    let src_offset = start_byte + row * width_bytes;
                    let dst_offset = row * w_bytes;

                    // 提取区域数据
                    for j in 0..w_bytes {
                        prev_data[dst_offset + j] = frame[src_offset + j];
                    }
                }

                if let Err(e) = EpdDisplay::partial_update(
                    &mut display,
                    region.x,
                    region.y,
                    region.width,
                    region.height,
                    &region.data,
                    &prev_data[..expected_len],
                ) {
                    error!("Failed to update region {}: {:?}", i, e);
                    DISPLAY.borrow(cs).replace(Some(display));
                    return Err(anyhow::anyhow!("Display partial update failed"));
                }

                // 更新帧缓存中的局部区域数据
                for row in 0..h {
                    let dst_offset = start_byte + row * width_bytes;
                    let src_offset = row * w_bytes;
                    for j in 0..w_bytes {
                        frame[dst_offset + j] = region.data[src_offset + j];
                    }
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
