pub mod config;
pub mod log;

use anyhow::Result;
use nihility_common::idea::{ChatIdea, Idea};
use nihility_common::inspiration::Inspiration;
use nihility_common::sender_memory_idea;
use tokio::sync::mpsc::Receiver;
use tracing::{info, warn};

pub async fn run(mut input_receiver: Receiver<Inspiration>) -> Result<()> {
    info!("Starting core thread");
    while let Some(entity) = input_receiver.recv().await {
        info!("{:?}", entity);
        match entity {
            Inspiration::ChatApp(chat_inspiration) => {
                sender_memory_idea(Idea::Memory(ChatIdea::Query(chat_inspiration))).await?;
            }
            Inspiration::Memory(memory_inspiration) => {
                warn!("Received inspiration: {:?}", memory_inspiration);
            }
        }
    }
    Ok(())
}
