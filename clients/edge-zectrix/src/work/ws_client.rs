use crate::display;
use crate::net::get_device_id;
use crate::storage::ServerConfig;
use alloc::boxed::Box;
use alloc::vec::Vec;
use anyhow::{bail, Result};
use core::net::{IpAddr, SocketAddr};
use core::str::FromStr;
use edge_http::io::client::Connection;
use edge_http::ws::{MAX_BASE64_KEY_LEN, MAX_BASE64_KEY_RESPONSE_LEN, NONCE_LEN};
use edge_nal_embassy::{Tcp, TcpBuffers, TcpSocket};
use edge_ws::{FrameHeader, FrameType};
use embassy_net::Stack;
use embassy_time::{Duration, Timer};
use esp_hal::rng::Rng;
use log::{error, info};
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
    mut socket: &mut TcpSocket<'a>,
    msg: &Message,
    rng: &mut Rng,
) -> Result<()> {
    let bytes = to_allocvec(msg).map_err(|e| anyhow::anyhow!("Serialize error: {:?}", e))?;
    info!("sending message: {:?}", bytes.as_slice());
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
async fn recv_message<'a>(mut socket: &mut TcpSocket<'a>, buf: &mut [u8]) -> Result<Message> {
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
pub async fn run_ws_client(stack: Stack<'static>, config: ServerConfig, rng: Rng) -> Result<()> {
    loop {
        match run_ws_client_once(&stack, &config, &mut rng.clone()).await {
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
    rng: &mut Rng,
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

    // 构建并发送设备信息
    send_message(&mut socket, &Message::DeviceInfo(build_device_info()), rng).await?;
    info!("DeviceInfo sent");

    // 消息接收循环（带心跳）
    // let mut last_heartbeat = embassy_time::Instant::now();

    loop {
        // 检查是否需要发送心跳
        // if last_heartbeat.elapsed().as_secs() >= HEARTBEAT_INTERVAL_SECS {
        //     let header = FrameHeader {
        //         frame_type: FrameType::Ping,
        //         payload_len: 0,
        //         mask_key: rng.random().into(),
        //     };
        //     header.send(&mut socket).await?;
        //     info!("Heartbeat sent");
        //     last_heartbeat = embassy_time::Instant::now();
        // }

        // 使用超时接收消息
        match recv_message(&mut socket, &mut msg_buf).await {
            Ok(msg) => match msg {
                Message::FullScreenUpdate(data) => {
                    info!("Received FullScreenUpdate: {}x{}", data.width, data.height);
                    if let Err(e) = display::full_screen_update(&data.data) {
                        error!("Full screen update failed: {:?}", e);
                    }
                }
                Message::IncrementalScreenUpdate(data) => {
                    info!(
                        "Received IncrementalScreenUpdate: {} regions",
                        data.regions.len()
                    );
                    if let Err(e) = display::incremental_screen_update(&data.regions) {
                        error!("Incremental screen update failed: {:?}", e);
                    }
                }
                _ => {
                    info!("Received unexpected message type: {:?}", msg);
                }
            },
            Err(e) => {
                // 连接错误，退出循环触发重连
                error!("Connection error: {:?}", e);
                break;
            }
        }
    }

    // 发送关闭帧
    let header = FrameHeader {
        frame_type: FrameType::Close,
        payload_len: 0,
        mask_key: rng.random().into(),
    };
    info!("Closing connection");
    header.send(&mut socket).await?;

    Ok(())
}
