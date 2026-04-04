use crate::error::*;
use nihility_edge_protocol::KeyCode;
use nihility_module_browser_control::func::press_key::PressKeyParam;
use nihility_module_browser_control::BrowserControl;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;

pub(crate) async fn start_key_handle(
    page_id: String,
    mut key_receiver: mpsc::UnboundedReceiver<KeyCode>,
    browser_control: Arc<RwLock<BrowserControl>>,
) -> Result<JoinHandle<Result<()>>> {
    let join_handle = tokio::spawn(async move {
        while let Some(key_code) = key_receiver.recv().await {
            browser_control
                .write()
                .await
                .press_key(PressKeyParam {
                    page_id: page_id.clone(),
                    key: key_code.to_browser_key(),
                })
                .await?;
        }
        Ok(())
    });
    Ok(join_handle)
}
