mod handle_api_resp_builder;
mod handle_message;
mod handle_meta;
mod handle_notice;
mod handle_request;

use crate::handle_api_resp_builder::handle_api_resp_builder;
use crate::handle_message::handle_message;
use crate::handle_meta::handle_meta;
use crate::handle_notice::handle_notice;
use crate::handle_request::handle_request;
use anyhow::Result;
use lazy_static::lazy_static;
use nihility_common::input::InputEntity;
use nihility_common::{register_chat_output_plugin, register_input_plugin};
use onebot_v11::Event;
pub use onebot_v11::connect::ws::WsConfig;
use onebot_v11::connect::ws::WsConnect;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tokio::sync::{Mutex, broadcast};
use tracing::{info, warn};
use uuid::Uuid;

lazy_static! {
    static ref CORE: Mutex<Option<NihilityChatInput>> = Mutex::new(None);
}

#[derive(Clone)]
pub struct NihilityChatInput {
    pub id: Option<Uuid>,
    pub bot_sender: Sender<InputEntity>,
    pub ws_connect: Arc<WsConnect>,
}

impl NihilityChatInput {
    pub async fn init(ws_config: &WsConfig) -> Result<()> {
        info!("Initializing Nihility Chat Input");
        let connect = WsConnect::new(ws_config.clone()).await?;
        let (tx, _) = broadcast::channel(10);
        let mut receiver = connect.subscribe().await;
        let mut core = Self {
            id: None,
            bot_sender: tx,
            ws_connect: connect,
        };
        core.id
            .replace(register_input_plugin(core.bot_sender.subscribe()).await?);
        CORE.lock().await.replace(core);

        tokio::spawn(async move {
            while let Ok(input_entity) = receiver.recv().await {
                match input_entity {
                    Event::Message(message) => handle_message(message).await,
                    Event::Meta(meta) => handle_meta(meta).await,
                    Event::Notice(notice) => handle_notice(notice).await,
                    Event::Request(request) => handle_request(request).await,
                    Event::ApiRespBuilder(api_resp_builder) => {
                        handle_api_resp_builder(api_resp_builder).await
                    }
                }
            }
        });

        tokio::spawn(async move {
            let mut output_receiver = register_chat_output_plugin().await;
            while let Ok(output_entity) = output_receiver.recv().await {
                warn!("Received chat should output {:?}", output_entity);
            }
        });

        Ok(())
    }

    fn sender_input(&self, input: InputEntity) -> Result<usize> {
        Ok(self.bot_sender.send(input)?)
    }

    fn get_id(&self) -> Result<Uuid> {
        match self.id {
            None => Err(anyhow::anyhow!("Id not initialized")),
            Some(id) => Ok(id),
        }
    }
}

pub(crate) async fn sender(input: InputEntity) -> Result<usize> {
    match CORE.lock().await.as_ref() {
        None => Err(anyhow::anyhow!("Core not initialized")),
        Some(core) => core.sender_input(input),
    }
}

pub(crate) async fn self_id() -> Result<Uuid> {
    match CORE.lock().await.as_ref() {
        None => Err(anyhow::anyhow!("Core not initialized")),
        Some(core) => core.get_id(),
    }
}
