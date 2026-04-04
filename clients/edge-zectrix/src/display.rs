mod epd;
mod epd_trait;

use crate::display::epd::EpdInterface;
use crate::display::epd_trait::EpdDisplay;
use crate::DISPLAY_CHANNEL;
use alloc::vec::Vec;
use anyhow::Result;
use epd::Display;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::peripherals::{GPIO10, GPIO11, GPIO12, GPIO13, GPIO8, GPIO9, SPI3};
use esp_hal::spi::master::Spi;
use esp_hal::spi::{BitOrder, Mode};
use esp_hal::time::Rate;
use log::{error, info};
use nihility_edge_protocol::{Message, UpdateRegion};

const WIDTH: u16 = 400;
const HEIGHT: u16 = 300;
const FRAME_SIZE: usize = (WIDTH as usize * HEIGHT as usize) / 8;

#[embassy_executor::task]
pub async fn display_task(
    busy: GPIO8<'static>,
    reset: GPIO9<'static>,
    dc: GPIO10<'static>,
    cs: GPIO11<'static>,
    spi3: SPI3<'static>,
    sck: GPIO12<'static>,
    sio: GPIO13<'static>,
) {
    let mut display =
        init_display(busy, reset, dc, cs, spi3, sck, sio).expect("Failed to init display");
    let mut frame_buf = [0u8; FRAME_SIZE];
    {
        let buffer = [0xFF_u8; FRAME_SIZE];
        if let Err(e) = full_screen_update(&mut display, &mut frame_buf, &buffer) {
            error!("Full screen update failed: {:?}", e);
        }
    }

    let receiver = DISPLAY_CHANNEL.receiver();
    loop {
        let message = receiver.receive().await;
        match message {
            Message::FullScreenUpdate(data) => {
                info!("Received FullScreenUpdate: {}x{}", data.width, data.height);
                if let Err(e) = full_screen_update(&mut display, &mut frame_buf, &data.data) {
                    error!("Full screen update failed: {:?}", e);
                }
            }
            Message::IncrementalScreenUpdate(data) => {
                info!(
                    "Received IncrementalScreenUpdate: {} regions",
                    data.regions.len()
                );
                if let Err(e) =
                    incremental_screen_update(&mut display, &mut frame_buf, &data.regions)
                {
                    error!("Incremental screen update failed: {:?}", e);
                }
            }
            _ => {}
        }
    }
}

pub fn init_display(
    busy: GPIO8<'static>,
    reset: GPIO9<'static>,
    dc: GPIO10<'static>,
    cs: GPIO11<'static>,
    spi3: SPI3<'static>,
    sck: GPIO12<'static>,
    sio: GPIO13<'static>,
) -> Result<Display> {
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

    Ok(display)
}

/// 全量屏幕更新
pub fn full_screen_update(
    display: &mut Display,
    frame_buf: &mut [u8; FRAME_SIZE],
    data: &[u8],
) -> Result<()> {
    info!("full screen update");
    if let Err(e) = display.init() {
        error!("Failed to init display: {:?}", e);
        return Err(anyhow::anyhow!("Display init failed"));
    }
    if let Err(e) = display.full_update(data) {
        error!("Failed to update display: {:?}", e);
        return Err(anyhow::anyhow!("Display update failed"));
    }

    // 更新帧缓存
    frame_buf.copy_from_slice(data);

    Ok(())
}

/// 增量屏幕更新
pub fn incremental_screen_update(
    display: &mut Display,
    frame_buf: &mut [u8; FRAME_SIZE],
    regions: &[UpdateRegion],
) -> Result<()> {
    info!("Incremental update: {} region(s)", regions.len());
    if let Err(e) = display.init() {
        error!("Failed to init display: {:?}", e);
        return Err(anyhow::anyhow!("Display init failed"));
    }

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
                prev_data[dst_offset + j] = frame_buf[src_offset + j];
            }
        }

        if let Err(e) = display.partial_update(
            region.x,
            region.y,
            region.width,
            region.height,
            &region.data,
            &prev_data[..expected_len],
        ) {
            error!("Failed to update region {}: {:?}", i, e);
            return Err(anyhow::anyhow!("Display partial update failed"));
        }

        // 更新帧缓存中的局部区域数据
        for row in 0..h {
            let dst_offset = start_byte + row * width_bytes;
            let src_offset = row * w_bytes;
            for j in 0..w_bytes {
                frame_buf[dst_offset + j] = region.data[src_offset + j];
            }
        }
    }
    Ok(())
}
