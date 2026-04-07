use crate::net::get_device_id;
use crate::storage::ServerConfig;
use crate::{FROM_SERVER_CHANNEL, TO_SERVER_CHANNEL};
use alloc::boxed::Box;
use alloc::vec::Vec;
use anyhow::{bail, Result};
use core::net::{IpAddr, SocketAddr};
use core::str::FromStr;
use edge_http::io::client::Connection;
use edge_http::ws::{MAX_BASE64_KEY_LEN, MAX_BASE64_KEY_RESPONSE_LEN, NONCE_LEN};
use edge_nal::TcpSplit;
use edge_nal_embassy::{Tcp, TcpBuffers, TcpSocketRead, TcpSocketWrite};
use edge_ws::{FrameHeader, FrameType};
use embassy_futures::select::select;
use embassy_net::Stack;
use embassy_time::{Duration, Timer};
use esp_hal::rng::Rng;
use log::{debug, error, info, warn};
use nihility_edge_protocol::{DeviceInfo, Message, ScreenConfig, ScreenRotation};
use postcard::{from_bytes, to_allocvec};

/// 重连间隔（秒）
const RETRY_INTERVAL_SECS: u64 = 5;

/// 构建设备信息
fn build_device_info() -> DeviceInfo {
    DeviceInfo {
        device_id: get_device_id().unwrap_or_default(),
        screen_width: 400,
        screen_height: 300,
        screen_refresh_interval: 100,
        screen_config: ScreenConfig {
            rotation: ScreenRotation::Rotate0,
            mirror_horizontal: false,
            #[cfg(feature = "ssd2683")]
            mirror_vertical: false,
            #[cfg(feature = "ssd1683")]
            mirror_vertical: true,
        },
    }
}

/// 发送消息（使用 rkyv 序列化）
async fn send_message<'a>(
    mut socket: &mut TcpSocketWrite<'a>,
    msg: &Message,
    rng: &Rng,
) -> Result<()> {
    let bytes = to_allocvec(msg).map_err(|e| anyhow::anyhow!("Serialize error: {:?}", e))?;
    debug!("sending message: {:?}", bytes.as_slice());
    let header = FrameHeader {
        frame_type: FrameType::Binary(false),
        payload_len: bytes.len() as _,
        mask_key: rng.random().into(),
    };
    header.send(&mut socket).await?;
    header.send_payload(&mut socket, bytes.as_slice()).await?;
    Ok(())
}

/// 接收并反序列化消息
async fn recv_message<'a>(mut socket: &mut TcpSocketRead<'a>, buf: &mut [u8]) -> Result<Message> {
    let header = FrameHeader::recv(&mut socket).await?;
    let payload = header.recv_payload(&mut socket, buf).await?;
    match header.frame_type {
        FrameType::Binary(_) => from_bytes::<Message>(payload)
            .map_err(|e| anyhow::anyhow!("Deserialize error: {:?}", e)),
        FrameType::Ping => {
            info!("Received Ping");
            Err(anyhow::anyhow!("Ping received"))
        }
        FrameType::Pong => {
            info!("Received Pong");
            Err(anyhow::anyhow!("Pong received"))
        }
        _ => {
            bail!("Unexpected frame type: {:?}", header.frame_type);
        }
    }
}

/// 运行 WebSocket 客户端（带重连机制）
pub async fn run_ws_client(stack: Stack<'static>, config: ServerConfig, rng: &Rng) -> Result<()> {
    loop {
        match run_ws_client_once(&stack, &config, &rng).await {
            Ok(()) => {
                info!("Connection closed normally");
                break;
            }
            Err(e) => {
                error!(
                    "Connection error: {:?}, reconnecting in {}s...",
                    e, RETRY_INTERVAL_SECS
                );
                Timer::after(Duration::from_secs(RETRY_INTERVAL_SECS)).await;
            }
        }
    }
    Ok(())
}

/// 单次 WebSocket 连接
async fn run_ws_client_once(
    stack: &Stack<'static>,
    config: &ServerConfig,
    rng: &Rng,
) -> Result<()> {
    let tcp_buf = Box::leak(Box::new_in(
        TcpBuffers::<3, 4096, 4096>::new(),
        esp_alloc::ExternalMemory,
    ));
    let tcp = Tcp::new(stack.clone(), tcp_buf);
    let mut ws_buf = Vec::new_in(esp_alloc::ExternalMemory);
    ws_buf.resize(20000, 0);
    let mut conn: Connection<Tcp> = Connection::new(
        &mut ws_buf,
        &tcp,
        SocketAddr::new(IpAddr::from_str(&config.host)?, config.port),
    );

    let nonce = [rng.random() as u8; NONCE_LEN];

    let mut buf = [0_u8; MAX_BASE64_KEY_LEN];
    conn.initiate_ws_upgrade_request(
        Some(&config.host),
        Some("foo.com"),
        "/ws/edge-device-control",
        None,
        &nonce,
        &mut buf,
    )
    .await?;
    conn.initiate_response().await?;

    let mut buf = [0_u8; MAX_BASE64_KEY_RESPONSE_LEN];
    if !conn.is_ws_upgrade_accepted(&nonce, &mut buf)? {
        bail!("WS upgrade failed");
    }

    conn.complete().await?;

    let (mut socket, mut msg_buf) = conn.release();

    info!("Connection upgraded to WS, starting traffic now");
    Timer::after(Duration::from_secs(1)).await;

    let (ws_tx, ws_rx) = socket.split();

    select(send_ws(ws_rx, &rng), receive_ws(ws_tx, &mut msg_buf)).await;

    Err(anyhow::anyhow!("Connection closed"))
}

async fn send_ws<'a>(mut ws_rx: TcpSocketWrite<'a>, rng: &Rng) -> Result<()> {
    send_message(&mut ws_rx, &Message::DeviceInfo(build_device_info()), rng).await?;
    info!("DeviceInfo sent");
    let to_server_receiver = TO_SERVER_CHANNEL.receiver();
    loop {
        let msg = to_server_receiver.receive().await;
        send_message(&mut ws_rx, &msg, rng).await?;
    }
}

async fn receive_ws<'a>(mut ws_tx: TcpSocketRead<'a>, msg_buf: &mut [u8]) -> Result<()> {
    let display_sender = FROM_SERVER_CHANNEL.sender();
    while let Ok(msg) = recv_message(&mut ws_tx, msg_buf).await {
        match msg {
            Message::FullScreenUpdate(data) => {
                display_sender.send(Message::FullScreenUpdate(data)).await
            }
            Message::IncrementalScreenUpdate(data) => {
                display_sender
                    .send(Message::IncrementalScreenUpdate(data))
                    .await
            }
            _ => {
                warn!("Received unexpected message type: {:?}", msg);
            }
        }
    }
    Ok(())
}
