mod core;

use crate::core::core_heat_flow;
use anyhow::Result;
use nihility_common::config::get_global_config;
use nihility_common::inspiration::Inspiration;
use rust_i18n::{i18n, set_locale};
use serde_json::Value;
use tokio::sync::mpsc::Receiver;
use tracing::{debug, info};

i18n!("locales", fallback = "zhs");

pub async fn run(mut inspiration_receiver: Receiver<Inspiration>) -> Result<()> {
    info!("Starting core thread");
    let local = get_global_config("local", Value::from("zhs")).await?;
    set_locale(local.as_str().unwrap_or("zhs"));
    while let Some(inspiration) = inspiration_receiver.recv().await {
        debug!("Receiver inspiration: {:?}", inspiration);
        match inspiration {
            Inspiration::External(external_inspiration) => {
                core_heat_flow(external_inspiration).await;
            }
            Inspiration::Internal(internal_inspiration) => {
                core_heat_flow(internal_inspiration).await;
            }
        }
    }
    Ok(())
}
