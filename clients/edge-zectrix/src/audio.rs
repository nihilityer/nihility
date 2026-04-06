mod es8311;

use alloc::vec::Vec;
use embassy_time::{Duration, Timer};
use embedded_hal_bus::i2c::RefCellDevice;
use esp_hal::dma::{DmaError, DmaTransferTxCircular};
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::i2c::master::I2c;
use esp_hal::i2s::master::{Channels, Config, DataFormat, Error, I2s, I2sRx, I2sTx};
use esp_hal::peripherals::{DMA_CH1, GPIO14, GPIO15, GPIO16, GPIO38, GPIO45, GPIO46, I2S0};
use esp_hal::time::Rate;
use esp_hal::{dma_buffers, Async, Blocking};
use log::{error, info};

#[embassy_executor::task]
pub async fn audio_task(
    i2s: I2S0<'static>,
    dma: DMA_CH1<'static>,
    mclk: GPIO14<'static>,
    ws: GPIO38<'static>,
    bclk: GPIO15<'static>,
    din: GPIO16<'static>,
    dout: GPIO45<'static>,
    pa: GPIO46<'static>,
    mut i2c: RefCellDevice<'static, I2c<'static, Blocking>>,
) {
    let _pa = Output::new(pa, Level::High, OutputConfig::default());

    let i2s = I2s::new(
        i2s,
        dma,
        Config::new_tdm_philips()
            .with_sample_rate(Rate::from_hz(16000))
            .with_data_format(DataFormat::Data16Channel16)
            .with_channels(Channels::MONO),
    )
    .expect("Failed to initialize I2S")
    .with_mclk(mclk)
    .into_async();
    let (rx_buffer, rx_descriptors, _, _) = dma_buffers!(4 * 4092, 0);
    // let (tx_buffer, tx_descriptors, _, _) = dma_buffers!(4 * 4092, 0);

    // let rx_buffer = Box::leak(Box::new_in(rx_buffer, esp_alloc::ExternalMemory));
    let i2s_rx = i2s
        .i2s_rx
        .with_bclk(bclk)
        .with_ws(ws)
        .with_din(din)
        .build(rx_descriptors);
    // let rx_transfer = i2s_rx
    //     .read_dma_circular_async(rx_buffer)
    //     .expect("Failed to create i2s read transfer");

    // let tx_buffer = Box::leak(Box::new_in(tx_buffer, esp_alloc::ExternalMemory));
    // let i2s_tx = Box::leak(Box::new_in(
    //     i2s.i2s_tx
    //         .with_bclk(NoPin)
    //         .with_ws(NoPin)
    //         .with_dout(dout)
    //         .build(tx_descriptors),
    //     esp_alloc::ExternalMemory,
    // ));
    // let tx_transfer = i2s_tx
    //     .write_dma_circular(tx_buffer)
    //     .expect("Failed to create i2s write transfer");
    // select(record(rx_transfer), play(tx_transfer)).await;
    record(rx_buffer, i2s_rx).await;
}

async fn record(rx_buf: &'static mut [u8; 16368], i2s_rx: I2sRx<'static, Async>) {
    let mut rx_transfer = i2s_rx
        .read_dma_circular_async(rx_buf)
        .expect("Failed to create i2s read transfer");
    let mut buf = Vec::new_in(esp_alloc::ExternalMemory);
    buf.resize(4092, 0);
    loop {
        match rx_transfer.pop(&mut buf).await {
            Ok(len) => {
                info!("Received {} bytes", len);
            }
            Err(e) => {
                if e != Error::DmaError(DmaError::Late) {
                    error!("Failed to receive buffer: {:?}", e);
                }
            }
        }
    }
}

async fn play(mut tx_transfer: DmaTransferTxCircular<'static, I2sTx<'static, Blocking>>) {
    loop {
        Timer::after(Duration::from_secs(1)).await
    }
}
