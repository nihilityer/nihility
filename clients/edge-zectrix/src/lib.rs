#![no_std]
#![no_main]
#![feature(allocator_api)]

extern crate alloc;

use crate::audio::audio_task;
use crate::display::display_task;
use crate::input::button_task;
use crate::net::wifi::{dhcp_task, net_task, run_ap_mode, run_client_mode, AP_SSID, GW_IP};
use crate::net::{get_device_id, MAX_RETRY_COUNT};
use crate::storage::{clear_credentials, init_storage, load_config};
use crate::work::config_server::run_config_server;
use crate::work::ws_client::run_ws_client;
use alloc::boxed::Box;
use alloc::format;
use anyhow::Result;
use core::cell::RefCell;
use core::net::Ipv4Addr;
use core::str::FromStr;
use embassy_executor::Spawner;
use embassy_net::{Config, DhcpConfig, StackResources};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use embedded_hal_bus::i2c::RefCellDevice;
use esp_hal::clock::CpuClock;
use esp_hal::i2c;
use esp_hal::i2c::master::I2c;
use esp_hal::rng::Rng;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_radio::wifi::{AccessPointConfig, ClientConfig};
use log::{error, info};
use nihility_edge_protocol::Message;
use smoltcp::wire::Ipv4Cidr;

mod audio;
mod display;
mod input;
mod net;
mod storage;
mod work;

static DISPLAY_CHANNEL: Channel<CriticalSectionRawMutex, Message, 8> = Channel::new();

pub async fn init(spawner: Spawner) -> Result<()> {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    esp_rtos::start(TimerGroup::new(peripherals.TIMG0).timer0);

    #[cfg(feature = "ssd2683")]
    {
        use esp_hal::gpio::{Level, Output, OutputConfig};
        let _pwr = Output::new(peripherals.GPIO17, Level::High, OutputConfig::default());
        let _epd_pwr = Output::new(peripherals.GPIO6, Level::High, OutputConfig::default());
    }

    let i2c = Box::leak(Box::new_in(
        RefCell::new(
            I2c::new(
                peripherals.I2C0,
                i2c::master::Config::default().with_frequency(Rate::from_khz(400)),
            )?
            .with_sda(peripherals.GPIO47)
            .with_scl(peripherals.GPIO48),
        ),
        esp_alloc::ExternalMemory,
    ));

    init_storage(peripherals.FLASH)?;

    spawner.spawn(display_task(
        peripherals.GPIO8,
        peripherals.GPIO9,
        peripherals.GPIO10,
        peripherals.GPIO11,
        peripherals.SPI3,
        peripherals.GPIO12,
        peripherals.GPIO13,
    ))?;

    spawner.spawn(button_task(
        peripherals.GPIO0,
        peripherals.GPIO39,
        peripherals.GPIO18,
    ))?;

    // spawner.spawn(audio_task(
    //     peripherals.I2S0,
    //     peripherals.DMA_CH1,
    //     peripherals.GPIO14,
    //     peripherals.GPIO38,
    //     peripherals.GPIO15,
    //     peripherals.GPIO16,
    //     peripherals.GPIO45,
    //     peripherals.GPIO46,
    //     RefCellDevice::new(i2c),
    // ))?;

    let config = load_config()?;

    let radio_init = Box::leak(Box::new(esp_radio::init()?));
    let (controller, interfaces) =
        esp_radio::wifi::new(radio_init, peripherals.WIFI, Default::default())?;

    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    if let Some(cfg) = config {
        // 有保存的配置，启动STA模式
        info!(
            "Found saved config: SSID={}, Server={}:{}",
            cfg.wifi.ssid, cfg.server.host, cfg.server.port
        );

        let sta_net_config = Config::dhcpv4(DhcpConfig::default());
        let (sta_stack, sta_runner) = embassy_net::new(
            interfaces.sta,
            sta_net_config,
            Box::leak(Box::new_in(
                StackResources::<6>::new(),
                esp_alloc::ExternalMemory,
            )),
            seed,
        );

        spawner.spawn(net_task(sta_runner))?;

        let client_config = ClientConfig::default()
            .with_ssid(cfg.wifi.ssid.clone())
            .with_password(cfg.wifi.password.clone());

        spawner.spawn(run_client_mode(controller, client_config))?;

        let mut retry_count = 0;
        while !sta_stack.is_link_up() || !sta_stack.is_config_up() {
            if retry_count >= MAX_RETRY_COUNT {
                error!("Retry limit reached");
                clear_credentials()?;
                Timer::after(Duration::from_secs(3)).await;
                esp_hal::system::software_reset();
            }
            retry_count += 1;
            Timer::after(Duration::from_secs(1)).await
        }

        // 启动 WebSocket 客户端连接服务器
        run_ws_client(sta_stack, cfg.server, &rng).await?;
    } else {
        // 没有凭证，启动AP模式
        info!("No saved WiFi credentials found, starting AP mode");
        let gw_ip = Ipv4Addr::from_str(GW_IP)?;
        let ap_net_config = Config::ipv4_static(embassy_net::StaticConfigV4 {
            address: Ipv4Cidr::new(gw_ip, 24),
            gateway: Some(gw_ip),
            dns_servers: Default::default(),
        });

        let (ap_stack, ap_runner) = embassy_net::new(
            interfaces.ap,
            ap_net_config,
            Box::leak(Box::new_in(
                StackResources::<6>::new(),
                esp_alloc::ExternalMemory,
            )),
            seed,
        );

        spawner.spawn(net_task(ap_runner))?;

        spawner.spawn(dhcp_task(ap_stack.clone()))?;

        let ap_config =
            AccessPointConfig::default().with_ssid(format!("{}{}", AP_SSID, get_device_id()?));

        spawner.spawn(run_ap_mode(controller, ap_config))?;

        run_config_server(ap_stack).await?;
    }
    Ok(())
}
