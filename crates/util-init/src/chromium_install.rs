use crate::error::*;
use chromiumoxide_fetcher::{BrowserFetcher, BrowserFetcherOptions};
use std::fs;
use tracing::info;

const CHROMIUM_PATH: &str = "chromium";

pub async fn chromium_install() -> Result<()> {
    info!("chromium install");
    if !fs::exists(CHROMIUM_PATH)? {
        fs::create_dir_all(CHROMIUM_PATH)?;
    }
    let options = BrowserFetcherOptions::builder()
        .with_path(CHROMIUM_PATH)
        .build()?;
    let fetcher = BrowserFetcher::new(options);
    let info = fetcher.fetch().await?;
    info!(
        "Chromium install finished at {}",
        info.executable_path.display()
    );
    Ok(())
}
