extern crate alloc;

use crate::net::get_device_id;
use crate::net::wifi::{Network, KNOWN_SSIDS};
use crate::storage::{save_config, DeviceConfig, ServerConfig, WifiCredentials};
use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use anyhow::{anyhow, Result};
use core::fmt::{Debug, Display};
use core::net::SocketAddr;
use core::str::FromStr;
use edge_http::io::server::{Connection, DefaultServer, Handler};
use edge_http::io::Error;
use edge_http::Method;
use edge_nal::TcpBind;
use edge_nal_embassy::{Tcp, TcpBuffers};
use embassy_net::Stack;
use embassy_time::{Duration, Timer};
use embedded_io_async::{Read, Write};
use log::{error, info};
use serde_json::json;

static WS_SERVER_ADDR: &str = "0.0.0.0:80";
const CHUNK_SIZE: usize = 2048; // 每次发送 2KB

/// 启动配网服务器的方法签名
pub async fn run_config_server(stack: Stack<'_>) -> Result<()> {
    while !stack.is_link_up() || !stack.is_config_up() {
        Timer::after(Duration::from_millis(100)).await
    }

    let buffer = Box::leak(Box::new_in(
        TcpBuffers::<3, 4096, 4096>::new(),
        esp_alloc::ExternalMemory,
    ));

    let mut server = DefaultServer::new();

    // 运行服务
    server
        .run(
            None,
            Tcp::new(stack, buffer)
                .bind(SocketAddr::from_str(WS_SERVER_ADDR)?)
                .await?,
            ConfigHandler,
        )
        .await
        .map_err(|e| anyhow!("Server Run Error: {:?}", e))?;

    Ok(())
}

struct ConfigHandler;

impl Handler for ConfigHandler {
    type Error<E>
        = Error<E>
    where
        E: Debug;

    async fn handle<T, const N: usize>(
        &self,
        _task_id: impl Display + Copy,
        conn: &mut Connection<'_, T, N>,
    ) -> core::result::Result<(), Self::Error<T::Error>>
    where
        T: Read + Write,
    {
        let headers = conn.headers()?;
        let method = headers.method;
        let path = headers.path;

        if method == Method::Get && path == "/" {
            // 使用 include_str! 引入 HTML 文件内容
            let html = include_str!("../../index.html");
            let bytes = html.as_bytes();

            conn.initiate_response(
                200,
                None,
                &[
                    ("Content-Type", "text/html"),
                    ("Content-Length", &bytes.len().to_string()),
                    ("Connection", "close"),
                ],
            )
            .await?;

            for chunk in bytes.chunks(CHUNK_SIZE) {
                conn.write_all(chunk).await?;
            }
        } else if method == Method::Post && path == "/save" {
            // 分配一个缓冲区读取 POST 提交的数据
            let mut body_buf = Vec::new_in(esp_alloc::ExternalMemory);
            body_buf.resize(1024, 0);
            let read_len = conn.read(&mut body_buf).await.unwrap_or(0);
            let body_str = core::str::from_utf8(&body_buf[..read_len]).unwrap_or("");

            // 极简解析 application/x-www-form-urlencoded
            let (mut ssid, mut password, mut server_host, mut server_port) =
                ("", "", "", "8080");
            for pair in body_str.split('&') {
                let mut parts = pair.splitn(2, '=');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    match key {
                        "ssid" => ssid = value,
                        "password" => password = value,
                        "server_host" => server_host = value,
                        "server_port" => server_port = value,
                        _ => {}
                    }
                }
            }

            // URL Decode：适配中文 SSID 和特殊字符密码
            let decoded_ssid = url_decode(ssid);
            let decoded_password = url_decode(password);
            let decoded_host = url_decode(server_host);
            let decoded_port = url_decode(server_port);

            // 解析端口号，默认 8080
            let port: u16 = decoded_port.parse().unwrap_or(8080);

            info!(
                "收到配置 - SSID: {}, 服务器: {}:{}",
                decoded_ssid,
                decoded_host,
                port
            );

            // 构建配置结构
            let config = DeviceConfig {
                wifi: WifiCredentials {
                    ssid: decoded_ssid,
                    password: decoded_password,
                },
                server: ServerConfig {
                    host: decoded_host,
                    port,
                },
            };

            // 尝试保存并返回响应
            match save_config(&config) {
                Ok(_) => {
                    conn.initiate_response(200, Some("OK"), &[]).await?;
                }
                Err(e) => {
                    error!("保存配置失败：{:?}", e);
                    conn.initiate_response(500, Some("Internal Server Error"), &[])
                        .await?;
                }
            }
            info!("config success");
            Timer::after(Duration::from_secs(5)).await;
            esp_hal::system::software_reset();
        } else if method == Method::Get && path == "/list" {
            let mut networks: Vec<Network> = critical_section::with(|cs| {
                KNOWN_SSIDS
                    .borrow_ref(cs)
                    .values()
                    .map(Clone::clone)
                    .collect()
            });
            info!("networks: {:?}", networks);
            networks.sort_by(|a, b| b.signal_strength.cmp(&a.signal_strength));
            let resp = serde_json::to_string(&networks).expect("JSON serialization failed");
            conn.initiate_response(200, Some("OK"), &[("Content-Type", "application/json")])
                .await?;
            let bytes = resp.as_bytes();
            for chunk in bytes.chunks(CHUNK_SIZE) {
                conn.write_all(chunk).await?;
            }
        } else if method == Method::Get && path == "/info" {
            let resp = json!({
                "device_id": get_device_id().expect("get device id"),
                "version": env!("CARGO_PKG_VERSION"),
            })
            .to_string();
            conn.initiate_response(200, Some("OK"), &[("Content-Type", "application/json")])
                .await?;
            conn.write_all(resp.as_bytes()).await?;
        } else {
            // 未知路径 404
            conn.initiate_response(404, Some("Not Found"), &[("Content-Type", "text/plain")])
                .await?;
            conn.write_all(b"404 Not Found").await?;
        }

        Ok(())
    }
}

/// 简单的 URL 解码器，用于解析带有中文或特殊字符的表单提交数据
fn url_decode(encoded: &str) -> String {
    let mut result = Vec::new();
    let bytes = encoded.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'%' => {
                // 如果发现 '%'，则解析后面的两位十六进制数
                if i + 2 < bytes.len() {
                    if let Ok(hex) = core::str::from_utf8(&bytes[i + 1..=i + 2]) {
                        if let Ok(val) = u8::from_str_radix(hex, 16) {
                            result.push(val);
                            i += 3;
                            continue;
                        }
                    }
                }
                // 如果解析失败，原样保留
                result.push(b'%');
                i += 1;
            }
            b'+' => {
                // 表单中的 '+' 代表空格
                result.push(b' ');
                i += 1;
            }
            b => {
                result.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&result).into_owned()
}
