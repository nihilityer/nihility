use crate::net::{get_device_id, SERVICE_PORT};
use alloc::boxed::Box;
use anyhow::{anyhow, Result};
use core::net::{Ipv4Addr, Ipv6Addr};
use edge_mdns::buf::VecBufAccess;
use edge_mdns::domain::base::Ttl;
use edge_mdns::host::{Service, ServiceAnswers};
use edge_mdns::io::{bind, Mdns, DEFAULT_SOCKET};
use edge_mdns::{host::Host, HostAnswersMdnsHandler};
use edge_nal::UdpSplit;
use edge_nal_embassy::{Udp, UdpBuffers};
use embassy_net::Stack;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::signal::Signal;
use esp_hal::rng::Rng;
use log::info;

#[embassy_executor::task]
pub async fn start_mdns(stack: Stack<'static>) {
    run_mdns(stack).await.expect("run mdns");
}

async fn run_mdns(stack: Stack<'_>) -> Result<()> {
    let device_id = &get_device_id()?;
    let (recv_buf, send_buf) = (
        VecBufAccess::<NoopRawMutex, 1500>::new(),
        VecBufAccess::<NoopRawMutex, 1500>::new(),
    );
    info!(
        "About to run an mDNS responder for client. It will be addressable using {device_id}.local"
    );

    let buffers = Box::leak(Box::new_in(
        UdpBuffers::<3, 2048, 2048, 10>::new(),
        esp_alloc::ExternalMemory,
    ));
    let unbound_socket = Udp::new(stack, buffers);
    let mut socket = bind(
        &unbound_socket,
        DEFAULT_SOCKET,
        Some(Ipv4Addr::UNSPECIFIED),
        Some(0),
    )
    .await?;
    let addr = stack
        .config_v4()
        .ok_or(anyhow!("net stack not config ip"))?
        .address
        .address();

    let (recv, send) = socket.split();

    let host = Host {
        hostname: device_id,
        ipv4: addr,
        ipv6: Ipv6Addr::UNSPECIFIED,
        ttl: Ttl::from_secs(60),
    };

    let service = Service {
        name: device_id,
        priority: 0,
        weight: 0,
        service: "_edge-device",
        protocol: "_tcp",
        port: SERVICE_PORT,
        service_subtypes: &[],
        txt_kvs: &[
            ("screen_width", "400"),
            ("screen_height", "300"),
            ("screen_refresh_interval", "100"),
        ],
    };

    let answers = ServiceAnswers::new(&host, &service);

    // A way to notify the mDNS responder that the data in `Host` had changed
    // We don't use it in this example, because the data is hard-coded
    let signal = Signal::<NoopRawMutex, _>::new();

    let mdns = Mdns::new(
        Some(Ipv4Addr::UNSPECIFIED),
        Some(0),
        recv,
        send,
        recv_buf,
        send_buf,
        Rng::new(),
        &signal,
    );

    mdns.run(HostAnswersMdnsHandler::new(&answers)).await?;
    Ok(())
}
