use crate::display::{full_screen_update, incremental_screen_update};
use crate::net::SERVICE_PORT;
use alloc::boxed::Box;
use alloc::format;
use alloc::vec::Vec;
use anyhow::Result;
use core::fmt::{Debug, Display};
use core::net::SocketAddr;
use core::str::FromStr;
use edge_http::io::server::{Connection, DefaultServer, Handler};
use edge_http::io::Error;
use edge_http::ws::MAX_BASE64_KEY_RESPONSE_LEN;
use edge_http::Method;
use edge_nal::TcpBind;
use edge_nal_embassy::{Tcp, TcpBuffers};
use edge_ws::{FrameHeader, FrameType};
use embassy_net::Stack;
use embedded_io_async::{Read, Write};
use log::{error, info, warn};
use nihility_edge_protocol::Message;

pub async fn run_ws_server(stack: Stack<'_>) -> Result<()> {
    info!("Running websocket server on {SERVICE_PORT}");
    let mut server = DefaultServer::new();

    let buffer = Box::leak(Box::new_in(
        TcpBuffers::<3, 4096, 4096>::new(),
        esp_alloc::ExternalMemory,
    ));
    server
        .run(
            None,
            Tcp::new(stack, buffer)
                .bind(SocketAddr::from_str(&format!("0.0.0.0:{}", SERVICE_PORT))?)
                .await?,
            WsHandler,
        )
        .await?;

    Ok(())
}

#[derive(Debug)]
enum WsHandlerError<C, W> {
    Connection(C),
    Ws(W),
}

impl<C, W> From<C> for WsHandlerError<C, W> {
    fn from(e: C) -> Self {
        Self::Connection(e)
    }
}

struct WsHandler;

impl Handler for WsHandler {
    type Error<E>
        = WsHandlerError<Error<E>, edge_ws::Error<E>>
    where
        E: Debug;

    async fn handle<T, const N: usize>(
        &self,
        _task_id: impl Display + Clone,
        conn: &mut Connection<'_, T, N>,
    ) -> Result<(), Self::Error<T::Error>>
    where
        T: Read + Write,
    {
        info!("Handling connection");
        let headers = conn.headers()?;

        if headers.method != Method::Get {
            conn.initiate_response(405, Some("Method Not Allowed"), &[])
                .await?;
        } else if headers.path != "/" {
            conn.initiate_response(404, Some("Not Found"), &[]).await?;
        } else if !conn.is_ws_upgrade_request()? {
            conn.initiate_response(200, Some("OK"), &[("Content-Type", "text/plain")])
                .await?;

            conn.write_all(b"Initiate WS Upgrade request to switch this connection to WS")
                .await?;
        } else {
            let mut buf = [0_u8; MAX_BASE64_KEY_RESPONSE_LEN];
            conn.initiate_ws_upgrade_response(&mut buf).await?;

            conn.complete().await?;

            info!("Connection upgraded to WS, starting a simple WS echo server now");

            // Now we have the TCP socket in a state where it can be operated as a WS connection
            // Run a simple WS echo server here

            let mut socket = conn.unbind()?;

            // 增大缓冲区以容纳完整屏幕数据 (400x300/8 = 15KB + 序列化开销)
            let mut buf = Vec::with_capacity_in(40960, esp_alloc::ExternalMemory);
            buf.resize(40960, 0);

            let mut timestamp: u64 = 0;
            loop {
                let header = FrameHeader::recv(&mut socket)
                    .await
                    .map_err(WsHandlerError::Ws)?;
                let payload = header
                    .recv_payload(&mut socket, &mut buf)
                    .await
                    .map_err(WsHandlerError::Ws)?;

                match header.frame_type {
                    FrameType::Text(_) => {
                        info!(
                            "Got {header}, with payload \"{}\"",
                            core::str::from_utf8(payload).unwrap()
                        );
                    }
                    FrameType::Binary(_) => {
                        // 反序列化消息
                        match rkyv::from_bytes::<Message, rkyv::rancor::Error>(payload) {
                            Ok(msg) => {
                                match msg {
                                    Message::FullScreenUpdate(screen_data) => {
                                        if screen_data.timestamp < timestamp {
                                            warn!(
                                                "new screen data timestamp is less than timestamp"
                                            );
                                            continue;
                                        }
                                        timestamp = screen_data.timestamp;
                                        // 执行全量屏幕更新
                                        if let Err(e) = full_screen_update(&screen_data.data) {
                                            info!("Failed to update screen: {:?}", e);
                                        }
                                    }
                                    Message::IncrementalScreenUpdate(screen_data) => {
                                        if screen_data.timestamp < timestamp {
                                            warn!(
                                                "new screen data timestamp is less than timestamp"
                                            );
                                            continue;
                                        }
                                        timestamp = screen_data.timestamp;
                                        // 执行增量屏幕更新
                                        if let Err(e) =
                                            incremental_screen_update(&screen_data.regions)
                                        {
                                            info!("Failed to update screen incrementally: {:?}", e);
                                        }
                                    }
                                    _ => {
                                        error!("unhandled message: {:?}", msg);
                                    }
                                }
                            }
                            Err(e) => {
                                info!("Failed to deserialize message: {:?}", e);
                            }
                        }
                    }
                    FrameType::Close => {
                        info!("Got {header}, client closed the connection cleanly");
                        break;
                    }
                    FrameType::Ping => {
                        info!("Got {header}, sending Pong");
                        // 响应 Ping 帧
                        let mut pong_header = header;
                        pong_header.mask_key = None;
                        pong_header.frame_type = FrameType::Pong;
                        pong_header
                            .send(&mut socket)
                            .await
                            .map_err(WsHandlerError::Ws)?;
                        pong_header
                            .send_payload(&mut socket, payload)
                            .await
                            .map_err(WsHandlerError::Ws)?;
                    }
                    _ => {
                        info!("Got {header}");
                    }
                }
            }
        }

        Ok(())
    }
}
