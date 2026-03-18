#![no_std]
#![no_main]
#![feature(allocator_api)]

extern crate alloc;

use crate::net::get_device_id;
use crate::net::mdns::start_mdns;
use crate::net::wifi::{dhcp_task, net_task, run_ap_mode, run_client_mode, AP_SSID, GW_IP};
use crate::storage::{init_storage, load_credentials};
use crate::work::config_server::run_config_server;
use crate::work::ws_server::run_ws_server;
use alloc::boxed::Box;
use alloc::format;
use anyhow::Result;
use core::net::Ipv4Addr;
use core::str::FromStr;
use embassy_executor::Spawner;
use embassy_net::{Config, DhcpConfig, StackResources};
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_radio::wifi::{AccessPointConfig, ClientConfig};
use log::info;
use smoltcp::wire::Ipv4Cidr;

mod net;
mod storage;
mod work;

pub async fn init(spawner: Spawner) -> Result<()> {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    esp_rtos::start(TimerGroup::new(peripherals.TIMG0).timer0);

    init_storage(peripherals.FLASH)?;

    let credentials = load_credentials()?;

    let radio_init = Box::leak(Box::new(esp_radio::init()?));
    let (controller, interfaces) =
        esp_radio::wifi::new(radio_init, peripherals.WIFI, Default::default())?;

    let rng = Rng::new();
    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    if let Some(creds) = credentials {
        // 有保存的凭证，启动STA模式
        info!("Found saved WiFi credentials: SSID={}", creds.ssid);

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
            .with_ssid(creds.ssid.clone())
            .with_password(creds.password.clone());

        spawner.spawn(run_client_mode(controller, client_config))?;

        while !sta_stack.is_link_up() || !sta_stack.is_config_up() {
            Timer::after(Duration::from_secs(1)).await
        }

        spawner.spawn(start_mdns(sta_stack.clone()))?;

        run_ws_server(sta_stack).await?;
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
