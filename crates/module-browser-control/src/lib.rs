pub mod error;
pub mod func;

use crate::error::*;
use chromiumoxide::handler::viewport::Viewport;
use chromiumoxide::{Browser, BrowserConfig};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BrowserControlConfig {
    pub viewport_width: u32,
    pub viewport_height: u32,
}

pub struct BrowserControl {
    browser: Browser,
}

impl BrowserControl {
    pub async fn init_from_file_config() -> Result<Self> {
        Self::init(nihility_config::get_config::<BrowserControlConfig>(env!(
            "CARGO_PKG_NAME"
        ))?)
        .await
    }

    pub async fn init(config: BrowserControlConfig) -> Result<Self> {
        let browser_config = BrowserConfig::builder()
            .viewport(Viewport {
                width: config.viewport_width,
                height: config.viewport_height,
                device_scale_factor: None,
                emulating_mobile: false,
                is_landscape: false,
                has_touch: false,
            })
            .new_headless_mode()
            .build()
            .map_err(BrowserControlError::BuildConfig)?;

        let (browser, mut handler) = Browser::launch(browser_config).await?;
        info!("Browser control initialized");
        tokio::spawn(async move {
            loop {
                while let Some(next) = handler.next().await {
                    if let Err(e) = next {
                        error!("Browser control handler error: {:?}", e);
                    }
                }
            }
        });
        Ok(BrowserControl { browser })
    }
}

impl Default for BrowserControlConfig {
    fn default() -> Self {
        Self {
            viewport_width: 400,
            viewport_height: 300,
        }
    }
}
