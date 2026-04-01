use crate::device::{DeviceInfo, ScreenConfig, ScreenRotation};
use crate::error::*;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tracing::info;

pub fn start_discovery(
    service_type: &str,
    tx: mpsc::UnboundedSender<(SocketAddr, DeviceInfo)>,
) -> Result<()> {
    let mdns = ServiceDaemon::new()?;
    let receiver = mdns.browse(service_type)?;

    tokio::spawn(async move {
        while let Ok(event) = receiver.recv_async().await {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    let device_id = info.get_fullname().to_string();
                    if let Some(addr) = info.get_addresses().iter().next() {
                        let socket_addr = SocketAddr::new(addr.to_ip_addr(), info.get_port());

                        // 尝试从 TXT 记录解析设备信息
                        let device_info = parse_device_info_from_txt(&info, &device_id).ok_or(
                            EdgeDeviceControlError::DeviceStatus(format!(
                                "device {} cannot get info",
                                device_id
                            )),
                        )?;

                        info!("Discovered device: {} at {}", device_id, socket_addr);
                        let _ = tx.send((socket_addr, device_info));
                    }
                }
                ServiceEvent::ServiceRemoved(_, full_name) => {
                    info!("Device removed: {}", full_name);
                }
                _ => {}
            }
        }
        Result::Ok(())
    });

    Ok(())
}

fn parse_device_info_from_txt(
    service_info: &mdns_sd::ResolvedService,
    device_id: &str,
) -> Option<DeviceInfo> {
    let properties = service_info.get_properties();

    let screen_width = properties
        .get("screen_width")?
        .val_str()
        .parse::<u16>()
        .ok()?;
    let screen_height = properties
        .get("screen_height")?
        .val_str()
        .parse::<u16>()
        .ok()?;
    let screen_refresh_interval = properties
        .get("screen_refresh_interval")?
        .val_str()
        .parse::<usize>()
        .ok()?;

    // 解析屏幕配置（带默认值）
    let rotation = properties
        .get("screen_rotation")
        .and_then(|v| v.val_str().parse::<u16>().ok())
        .and_then(|r| match r {
            0 => Some(ScreenRotation::Rotate0),
            90 => Some(ScreenRotation::Rotate90),
            180 => Some(ScreenRotation::Rotate180),
            270 => Some(ScreenRotation::Rotate270),
            _ => None,
        })
        .unwrap_or(ScreenRotation::Rotate0);

    let mirror_horizontal = properties
        .get("screen_mirror_h")
        .and_then(|v| v.val_str().parse::<bool>().ok())
        .unwrap_or(false);

    let mirror_vertical = properties
        .get("screen_mirror_v")
        .and_then(|v| v.val_str().parse::<bool>().ok())
        .unwrap_or(false);

    let screen_config = ScreenConfig {
        rotation,
        mirror_horizontal,
        mirror_vertical,
    };

    Some(DeviceInfo {
        device_id: device_id.to_string(),
        screen_width,
        screen_height,
        screen_refresh_interval,
        screen_config,
    })
}
