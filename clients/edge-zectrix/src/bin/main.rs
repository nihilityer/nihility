#![no_std]
#![no_main]
#![feature(allocator_api)]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

extern crate alloc;

use alloc::boxed::Box;
use alloc::format;
use core::cell::RefCell;
use core::net::Ipv4Addr;
use embassy_executor::Spawner;
use embassy_net::{Config, DhcpConfig, StackResources};
use embassy_time::{Duration, Timer};
use embedded_hal_bus::i2c::RefCellDevice;
use esp_hal::{i2c, interrupt::software::SoftwareInterruptControl, system::Stack};
use nihility_edge_zectrix::audio::audio_task;
use nihility_edge_zectrix::display::display_task;
use nihility_edge_zectrix::input::button_task;
use nihility_edge_zectrix::storage::{clear_credentials, init_storage, load_config};
use static_cell::StaticCell;

use core::str::FromStr;
#[allow(unused)]
use esp_alloc as _;
#[allow(unused)]
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::I2c;
use esp_hal::rng::Rng;
use esp_hal::time::Rate;
use esp_hal::timer::timg::TimerGroup;
use esp_radio::wifi::{AccessPointConfig, ClientConfig};
use esp_rtos::embassy::Executor;
use log::{error, info};
use nihility_edge_zectrix::net::wifi::{
    dhcp_task, net_task, run_ap_mode, run_client_mode, AP_SSID, GW_IP,
};
use nihility_edge_zectrix::net::{get_device_id, MAX_RETRY_COUNT};
use nihility_edge_zectrix::work::config_server::run_config_server;
use nihility_edge_zectrix::work::ws_client::run_ws_client;
use smoltcp::wire::Ipv4Cidr;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);
    esp_alloc::psram_allocator!(peripherals.PSRAM, esp_hal::psram);

    static APP_CORE_STACK: StaticCell<Stack<8192>> = StaticCell::new();
    let app_core_stack = APP_CORE_STACK.init(Stack::new());

    let sw_int = SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);

    esp_rtos::start(TimerGroup::new(peripherals.TIMG0).timer0);

    #[cfg(feature = "ssd2683")]
    esp_rtos::start_second_core(
        peripherals.CPU_CTRL,
        sw_int.software_interrupt0,
        sw_int.software_interrupt1,
        app_core_stack,
        move || {
            static EXECUTOR: StaticCell<Executor> = StaticCell::new();
            let executor = EXECUTOR.init(Executor::new());
            executor.run(|spawner| {
                let i2c = Box::leak(Box::new_in(
                    RefCell::new(
                        I2c::new(
                            peripherals.I2C0,
                            i2c::master::Config::default().with_frequency(Rate::from_khz(400)),
                        )
                        .expect("failed to create i2c")
                        .with_sda(peripherals.GPIO47)
                        .with_scl(peripherals.GPIO48),
                    ),
                    esp_alloc::ExternalMemory,
                ));
                spawner
                    .spawn(audio_task(
                        peripherals.I2S0,
                        peripherals.DMA_CH1,
                        peripherals.GPIO14,
                        peripherals.GPIO38,
                        peripherals.GPIO15,
                        peripherals.GPIO16,
                        peripherals.GPIO45,
                        RefCellDevice::new(i2c),
                    ))
                    .expect("failed to spawn audio_task");
            });
        },
    );

    #[cfg(feature = "ssd2683")]
    {
        use esp_hal::gpio::{Level, Output, OutputConfig};
        let _pwr = Output::new(peripherals.GPIO17, Level::High, OutputConfig::default());
        let _epd_pwr = Output::new(peripherals.GPIO6, Level::High, OutputConfig::default());
        let _pa = Output::new(peripherals.GPIO46, Level::High, OutputConfig::default());
        let _amp_pwr = Output::new(peripherals.GPIO42, Level::High, OutputConfig::default());
    }

    init_storage(peripherals.FLASH).expect("failed to initialize storage");

    spawner
        .spawn(display_task(
            peripherals.GPIO8,
            peripherals.GPIO9,
            peripherals.GPIO10,
            peripherals.GPIO11,
            peripherals.SPI3,
            peripherals.GPIO12,
            peripherals.GPIO13,
        ))
        .expect("failed to spawn display_task");

    #[cfg(feature = "ssd2683")]
    spawner
        .spawn(button_task(
            peripherals.GPIO0,
            peripherals.GPIO39,
            peripherals.GPIO18,
        ))
        .expect("failed to spawn button_task");

    let config = load_config().expect("failed to load configuration");

    let radio_init = Box::leak(Box::new(esp_radio::init().expect("failed to init radio")));
    let (controller, interfaces) =
        esp_radio::wifi::new(radio_init, peripherals.WIFI, Default::default())
            .expect("failed to initialize wifi");

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

        spawner
            .spawn(net_task(sta_runner))
            .expect("failed to spawn network_task");

        let client_config = ClientConfig::default()
            .with_ssid(cfg.wifi.ssid.clone())
            .with_password(cfg.wifi.password.clone());

        spawner
            .spawn(run_client_mode(controller, client_config))
            .expect("failed to spawn client_mode_task");

        let mut retry_count = 0;
        while !sta_stack.is_link_up() || !sta_stack.is_config_up() {
            if retry_count >= MAX_RETRY_COUNT {
                error!("Retry limit reached");
                clear_credentials().expect("failed to clear credentials");
                Timer::after(Duration::from_secs(3)).await;
                esp_hal::system::software_reset();
            }
            retry_count += 1;
            Timer::after(Duration::from_secs(1)).await
        }

        // 启动 WebSocket 客户端连接服务器
        run_ws_client(sta_stack, cfg.server, &rng)
            .await
            .expect("failed to run ws_client");
    } else {
        // 没有凭证，启动AP模式
        info!("No saved WiFi credentials found, starting AP mode");
        let gw_ip = Ipv4Addr::from_str(GW_IP).expect("failed to parse GW IP");
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

        spawner
            .spawn(net_task(ap_runner))
            .expect("failed to spawn network_task");

        spawner
            .spawn(dhcp_task(ap_stack.clone()))
            .expect("failed to spawn dhcp_task");

        let ap_config = AccessPointConfig::default().with_ssid(format!(
            "{}{}",
            AP_SSID,
            get_device_id().expect("failed to get device id")
        ));

        spawner
            .spawn(run_ap_mode(controller, ap_config))
            .expect("failed to spawn ap_mode_task");

        run_config_server(ap_stack)
            .await
            .expect("failed to run config_server");
    }

    loop {
        Timer::after(Duration::from_secs(10)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0/examples
}
