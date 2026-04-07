mod es8311;

use crate::audio::es8311::{ClockConfig, Es8311, MicGain, Resolution};
use crate::TO_SERVER_CHANNEL;
use alloc::vec::Vec;
use embassy_time::{Duration, Timer};
use embedded_hal_bus::i2c::RefCellDevice;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::i2c::master::I2c;
use esp_hal::i2s::master::{Channels, Config, DataFormat, I2s, I2sRx};
use esp_hal::peripherals::{DMA_CH1, GPIO14, GPIO15, GPIO16, GPIO38, GPIO45, GPIO46, I2S0};
use esp_hal::time::Rate;
use esp_hal::{dma_buffers, Async, Blocking};
use log::error;
use nihility_edge_protocol::{AudioData, Message};

const CHUNK_SIZE: usize = 512 * 2;
const STEREO_SAMPLES_PER_READ: usize = 256;
const MONO_SAMPLES_PER_READ: usize = STEREO_SAMPLES_PER_READ / 2;

#[embassy_executor::task]
pub async fn audio_task(
    i2s: I2S0<'static>,
    dma: DMA_CH1<'static>,
    mclk: GPIO14<'static>,
    ws: GPIO38<'static>,
    bclk: GPIO15<'static>,
    din: GPIO16<'static>,
    _dout: GPIO45<'static>,
    pa: GPIO46<'static>,
    mut i2c: RefCellDevice<'static, I2c<'static, Blocking>>,
) {
    let _pa = Output::new(pa, Level::High, OutputConfig::default());

    let codec = Es8311::new(0x18);
    let clk_cfg = ClockConfig {
        mclk_inverted: false,
        sclk_inverted: false,
        mclk_from_mclk_pin: true,
        mclk_frequency: 4_096_000, // 12.288 MHz
        sample_frequency: 16_000,
    };
    codec
        .init(&mut i2c, &clk_cfg, Resolution::Bits16, Resolution::Bits16)
        .expect("init es8311 failed");
    codec
        .microphone_config(&mut i2c, true)
        .expect("microphone config failed");
    codec
        .microphone_gain_set(&mut i2c, MicGain::Gain24dB)
        .expect("microphone gain set failed");
    codec
        .volume_set(&mut i2c, 80, None)
        .expect("volume set failed");
    codec.mute(&mut i2c, true).expect("mute failed");

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
    let (_, rx_descriptors, _, _tx_descriptors) = dma_buffers!(4 * 4092);

    let i2s_rx = i2s
        .i2s_rx
        .with_bclk(bclk)
        .with_ws(ws)
        .with_din(din)
        .build(rx_descriptors);
    Timer::after(Duration::from_millis(10)).await;

    // let tx_buffer = Box::leak(Box::new_in(tx_buffer, esp_alloc::ExternalMemory));
    // let i2s_tx = Box::leak(Box::new_in(
    //     i2s.i2s_tx
    //         .with_bclk(NoPin)
    //         .with_ws(NoPin)
    //         .with_dout(dout)
    //         .build(tx_descriptors),
    //     esp_alloc::ExternalMemory,
    // ));
    // select(record(rx_transfer), play(tx_transfer)).await;
    record(i2s_rx).await;
}

async fn record(mut i2s_rx: I2sRx<'static, Async>) {
    Timer::after(Duration::from_secs(10)).await;
    let mut i2s_buffer = [0u8; STEREO_SAMPLES_PER_READ * 2];
    let mut mono_i16 = [0i16; MONO_SAMPLES_PER_READ];
    let mut mono_f32 = [0f32; MONO_SAMPLES_PER_READ];
    let mut accum_buffer = Vec::with_capacity_in(CHUNK_SIZE, esp_alloc::ExternalMemory);
    let sender = TO_SERVER_CHANNEL.sender();
    loop {
        if let Err(e) = i2s_rx.read_dma_async(&mut i2s_buffer).await {
            error!("I2S read error: {:?}", e);
            Timer::after(Duration::from_millis(10)).await;
            continue;
        }
        let stereo = unsafe {
            core::slice::from_raw_parts(i2s_buffer.as_ptr() as *const i16, STEREO_SAMPLES_PER_READ)
        };
        for (i, chunk) in stereo.chunks(2).enumerate() {
            mono_i16[i] = ((chunk[0] as i32 + chunk[1] as i32) / 2) as i16;
        }
        for (i, &s) in mono_i16.iter().enumerate() {
            mono_f32[i] = s as f32 / 32768.0;
        }
        accum_buffer.extend_from_slice(&mono_f32[..MONO_SAMPLES_PER_READ]);
        if accum_buffer.len() >= CHUNK_SIZE {
            sender
                .send(Message::AudioData(AudioData {
                    audio_data: accum_buffer.to_vec(),
                    timestamp: 0,
                }))
                .await;
            accum_buffer.clear();
        }
    }
}

// async fn play(mut tx_transfer: DmaTransferTxCircular<'static, I2sTx<'static, Blocking>>) {
//     loop {
//         Timer::after(Duration::from_secs(1)).await
//     }
// }
