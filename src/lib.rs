pub mod config;
pub mod log;

use anyhow::Result;
use nihility_common::input::InputEntity;
use nihility_common::output::{Output, OutputEntity};
use nihility_common::sender_chat_output;
use tokio::sync::mpsc::Receiver;
use tracing::info;

pub async fn run(mut input_receiver: Receiver<InputEntity>) -> Result<()> {
    info!("Starting core thread");
    while let Some(entity) = input_receiver.recv().await {
        info!("{:?}", entity);
        sender_chat_output(OutputEntity {
            context: entity.context.clone(),
            input: Output::Text("收到消息了".to_string()),
        })
        .await?;
    }
    Ok(())
}
