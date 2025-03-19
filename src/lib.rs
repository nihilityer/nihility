pub mod log;

use anyhow::Result;
use nihility_common::output::{Output, OutputEntity};
use nihility_common::{init_input_sender, sender_chat_output};
use tracing::info;

pub async fn run() -> Result<()> {
    info!("Starting core thread");
    let mut input_receiver = init_input_sender(20).await;
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
