use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::cell::RefCell;
use core::str::FromStr;
use critical_section::Mutex;
use embassy_net::{Runner, Stack};
use embassy_time::{Duration, Timer};
use esp_radio::wifi::{
    AccessPointConfig, ClientConfig, ScanConfig, ScanMethod, WifiController, WifiDevice, WifiEvent,
};
use log::{error, info};
use serde::{Deserialize, Serialize};

pub const AP_SSID: &str = "nihility-edge-";
pub const GW_IP: &str = "192.168.7.1";

pub static KNOWN_SSIDS: Mutex<RefCell<BTreeMap<String, Network>>> =
    Mutex::new(RefCell::new(BTreeMap::new()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    pub ssid: String,
    pub channel: u8,
    pub signal_strength: i8,
    pub auth_method: Option<AuthMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    /// No authentication (open network).
    None,

    /// Wired Equivalent Privacy (WEP) authentication.
    Wep,

    /// Wi-Fi Protected Access (WPA) authentication.
    Wpa,

    /// Wi-Fi Protected Access 2 (WPA2) Personal authentication (default).
    Wpa2Personal,

    /// WPA/WPA2 Personal authentication (supports both).
    WpaWpa2Personal,

    /// WPA2 Enterprise authentication.
    Wpa2Enterprise,

    /// WPA3 Personal authentication.
    Wpa3Personal,

    /// WPA2/WPA3 Personal authentication (supports both).
    Wpa2Wpa3Personal,

    /// WLAN Authentication and Privacy Infrastructure (WAPI).
    WapiPersonal,
}

#[embassy_executor::task]
pub async fn run_ap_mode(mut controller: WifiController<'static>, ap_config: AccessPointConfig) {
    loop {
        if !matches!(controller.is_started(), Ok(true)) {
            controller
                .set_config(&esp_radio::wifi::ModeConfig::ApSta(
                    ClientConfig::default().with_scan_method(ScanMethod::AllChannels),
                    ap_config.clone(),
                ))
                .expect("Failed to set config");
            info!("Starting wifi");
            controller.start_async().await.unwrap();
            info!("Wifi started!");
        }
        let scan_config = ScanConfig::default();

        match controller.scan_with_config_async(scan_config).await {
            Ok(networks) => {
                info!("Scan finished, found {} networks", networks.len());

                critical_section::with(|cs| {
                    let mut known_ssids = KNOWN_SSIDS.borrow_ref_mut(cs);

                    for ap in networks {
                        let ssid_str = ap.ssid.as_str();
                        if ssid_str.is_empty() {
                            continue;
                        }
                        if !known_ssids.contains_key(ssid_str) {
                            info!(
                                "New Network: [{}] Signal: {}dBm",
                                ssid_str, ap.signal_strength
                            );
                        }
                        known_ssids.insert(
                            ssid_str.to_string(),
                            Network {
                                ssid: ap.ssid,
                                channel: ap.channel,
                                signal_strength: ap.signal_strength,
                                auth_method: ap.auth_method.map(|x| x.into()),
                            },
                        );
                    }
                });
            }
            Err(e) => {
                error!("Scan failed: {:?}", e);
            }
        }
        Timer::after(Duration::from_secs(10)).await;
    }
}

#[embassy_executor::task]
pub async fn run_client_mode(mut controller: WifiController<'static>, client_config: ClientConfig) {
    controller
        .set_config(&esp_radio::wifi::ModeConfig::Client(client_config))
        .expect("Failed to set config");
    info!("Starting wifi in client mode");
    controller.start_async().await.expect("Start wifi failed");
    info!("Wifi client mode started!");
    loop {
        if matches!(controller.is_started(), Ok(true)) {
            info!("About to connect...");

            match controller.connect_async().await {
                Ok(_) => {
                    // wait until we're no longer connected
                    controller.wait_for_event(WifiEvent::StaDisconnected).await;
                    info!("Station disconnected");
                }
                Err(e) => {
                    info!("Failed to connect to wifi: {e:?}");
                    Timer::after(Duration::from_millis(5000)).await
                }
            }
        } else {
            return;
        }
    }
}

#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}

#[embassy_executor::task]
pub async fn dhcp_task(stack: Stack<'static>) {
    use core::net::{Ipv4Addr, SocketAddrV4};

    use edge_dhcp::{
        io::{self, DEFAULT_SERVER_PORT},
        server::{Server, ServerOptions},
    };
    use edge_nal::UdpBind;
    use edge_nal_embassy::{Udp, UdpBuffers};
    info!("Setting up DHCP server");

    let ip = Ipv4Addr::from_str(GW_IP).expect("dhcp task failed to parse gw ip");

    let mut buf = Vec::with_capacity_in(1500, esp_alloc::ExternalMemory);
    buf.resize(1500, 0);

    let mut gw_buf = [Ipv4Addr::UNSPECIFIED];

    let buffers = UdpBuffers::<3, 1024, 1024, 10>::new();
    let unbound_socket = Udp::new(stack, &buffers);
    let mut bound_socket = unbound_socket
        .bind(core::net::SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            DEFAULT_SERVER_PORT,
        )))
        .await
        .unwrap();

    loop {
        _ = io::server::run(
            &mut Server::<_, 64>::new_with_et(ip),
            &ServerOptions::new(ip, Some(&mut gw_buf)),
            &mut bound_socket,
            &mut buf,
        )
        .await
        .inspect_err(|e| error!("DHCP server error: {e:?}"));
        Timer::after(Duration::from_millis(500)).await;
    }
}

impl From<esp_radio::wifi::AuthMethod> for AuthMethod {
    fn from(value: esp_radio::wifi::AuthMethod) -> Self {
        match value {
            esp_radio::wifi::AuthMethod::None => AuthMethod::None,
            esp_radio::wifi::AuthMethod::Wep => AuthMethod::Wep,
            esp_radio::wifi::AuthMethod::Wpa => AuthMethod::Wpa,
            esp_radio::wifi::AuthMethod::Wpa2Personal => AuthMethod::Wpa2Personal,
            esp_radio::wifi::AuthMethod::WpaWpa2Personal => AuthMethod::WpaWpa2Personal,
            esp_radio::wifi::AuthMethod::Wpa2Enterprise => AuthMethod::Wpa2Enterprise,
            esp_radio::wifi::AuthMethod::Wpa3Personal => AuthMethod::Wpa3Personal,
            esp_radio::wifi::AuthMethod::Wpa2Wpa3Personal => AuthMethod::Wpa2Wpa3Personal,
            esp_radio::wifi::AuthMethod::WapiPersonal => AuthMethod::WapiPersonal,
            _ => AuthMethod::None,
        }
    }
}
