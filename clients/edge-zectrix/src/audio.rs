mod es8311;

use crate::audio::es8311::{
    Address, Config as Es8311Config, Es8311, Gain, MclkFreq, Resolution, SampleFreq,
};
use crate::TO_SERVER_CHANNEL;
use alloc::vec::Vec;
use embassy_time::{Duration, Timer};
use embedded_hal_bus::i2c::RefCellDevice;
use esp_hal::delay::Delay;
use esp_hal::i2c::master::I2c;
use esp_hal::i2s::master::{Channels, DataFormat, Error, I2s, I2sRx};
use esp_hal::peripherals::{DMA_CH1, GPIO14, GPIO15, GPIO16, GPIO38, GPIO45, I2S0};
use esp_hal::time::Rate;
use esp_hal::{dma_buffers, Async, Blocking};
use log::{error, info};
use nihility_edge_protocol::{AudioData, Message};

const CHUNK_SIZE: usize = 512 * 16;

#[embassy_executor::task]
pub async fn audio_task(
    i2s: I2S0<'static>,
    dma: DMA_CH1<'static>,
    mclk: GPIO14<'static>,
    ws: GPIO38<'static>,
    bclk: GPIO15<'static>,
    din: GPIO16<'static>,
    _dout: GPIO45<'static>,
    i2c: RefCellDevice<'static, I2c<'static, Blocking>>,
) {
    let mut codec = Es8311::new(i2c, Address::Primary);
    let config = Es8311Config {
        sample_frequency: SampleFreq::Freq16KHz,
        mclk: Some(MclkFreq::Freq4096KHz),
        bits_per_sample: Resolution::Resolution16,
        mclk_inverted: false,
        sclk_inverted: false,
    };
    codec.init(&config).expect("init es8311 failed");
    codec
        .set_mic_gain(Gain::Gain36db)
        .expect("microphone gain set failed");
    Timer::after(Duration::from_secs(1)).await;

    let i2s = I2s::new(
        i2s,
        dma,
        esp_hal::i2s::master::Config::new_tdm_philips()
            .with_sample_rate(Rate::from_hz(16000))
            .with_data_format(DataFormat::Data16Channel16)
            .with_channels(Channels::LEFT),
    )
    .expect("Failed to initialize I2S")
    .with_mclk(mclk)
    .into_async();
    let (rx_buffer, rx_descriptors, _, _tx_descriptors) = dma_buffers!(4 * 4092);

    let i2s_rx = i2s
        .i2s_rx
        .with_bclk(bclk)
        .with_ws(ws)
        .with_din(din)
        .build(rx_descriptors);
    Timer::after(Duration::from_secs(1)).await;
    record(rx_buffer, i2s_rx).await;
}

async fn record(rx_buffer: &'static mut [u8; 16368], i2s_rx: I2sRx<'static, Async>) {
    Timer::after(Duration::from_secs(5)).await;
    info!("Recording");
    let mut i2s_buffer = [0u8; 4092 * 2];
    let mut accum_buffer = Vec::with_capacity_in(CHUNK_SIZE, esp_alloc::ExternalMemory);
    let sender = TO_SERVER_CHANNEL.sender();
    let mut transfer = i2s_rx
        .read_dma_circular_async(rx_buffer)
        .expect("read_dma_circular failed");
    let mut debug_output = true;
    loop {
        match transfer.pop(&mut i2s_buffer).await {
            Ok(len) => {
                if debug_output {
                    debug_output = false;
                }
                accum_buffer.extend_from_slice(
                    &i2s_buffer[..len]
                        .chunks_exact(2)
                        .map(|c| i16::from_le_bytes([c[0], c[1]]))
                        .map(|x| x as f32 / i16::MAX as f32)
                        .collect::<Vec<f32>>(),
                );

                while accum_buffer.len() >= CHUNK_SIZE {
                    sender
                        .send(Message::AudioData(AudioData {
                            audio_data: accum_buffer[..CHUNK_SIZE].to_vec(),
                        }))
                        .await;

                    if accum_buffer.len() > CHUNK_SIZE {
                        let remaining = accum_buffer[CHUNK_SIZE..].to_vec();
                        accum_buffer.clear();
                        accum_buffer.extend_from_slice(&remaining);
                    } else {
                        accum_buffer.clear();
                    }
                }
            }
            Err(e) => {
                error!("I2S read error: {:?}", e);
                Timer::after(Duration::from_millis(10)).await;
                continue;
            }
        }
    }
}
