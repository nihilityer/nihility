use anyhow::{Result, anyhow};
use lazy_static::lazy_static;
use nihility_common::model::get_chat_completion;
use rust_i18n::t;
use tokio::sync::Mutex;
use tracing::info;

lazy_static! {
    static ref THINK: Mutex<String> = Mutex::new(String::new());
}

pub async fn think(summary: String) -> Result<String> {
    let update_system_prompt = format!("{}\n{}", t!("prompt.update_think"), THINK.lock().await);
    match get_chat_completion(update_system_prompt, summary.to_string())
        .await?
        .get("text")
    {
        None => Err(anyhow!("Model Output Error")),
        Some(think) => {
            info!("Update think: {}", think);
            *THINK.lock().await = think.to_string();
            Ok(think.to_string())
        }
    }
}
