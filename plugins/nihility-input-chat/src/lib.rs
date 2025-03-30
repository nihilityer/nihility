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
use nihility_common::config::{get_config, get_global_config};
use nihility_common::inspiration::Inspiration;
use nihility_common::{register_intention_receiver_plugin, register_inspiration_plugin};
use onebot_v11::Event;
pub use onebot_v11::connect::ws::WsConfig;
use onebot_v11::connect::ws::WsConnect;
use std::sync::Arc;
use rust_i18n::{i18n, set_locale};
use serde_json::Value;
use tokio::sync::broadcast::Sender;
use tokio::sync::{Mutex, broadcast};
use tracing::{info, warn};

i18n!("locales", fallback = "zhs");

lazy_static! {
    static ref CORE: Mutex<Option<NihilityChatInput>> = Mutex::new(None);
}

#[derive(Clone)]
pub struct NihilityChatInput {
    pub bot_sender: Sender<Inspiration>,
    pub ws_connect: Arc<WsConnect>,
}

impl NihilityChatInput {
    pub async fn init() -> Result<()> {
        info!("Initializing Nihility Chat Input");
        let config = get_config::<WsConfig>(env!("CARGO_PKG_NAME").to_string()).await?;
        let local = get_global_config("local", Value::from("zhs")).await?;
        set_locale(local.as_str().unwrap_or("zhs"));
        let connect = WsConnect::new(config).await?;
        let (tx, _) = broadcast::channel(10);
        let mut receiver = connect.subscribe().await;
        let core = Self {
            bot_sender: tx,
            ws_connect: connect,
        };
        register_inspiration_plugin(core.bot_sender.subscribe()).await?;
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
            let mut output_receiver = register_intention_receiver_plugin().await;
            while let Ok(output_entity) = output_receiver.recv().await {
                warn!("Received chat should output {:?}", output_entity);
            }
        });

        Ok(())
    }

    fn sender_input(&self, input: Inspiration) -> Result<usize> {
        Ok(self.bot_sender.send(input)?)
    }
}

pub(crate) async fn sender(input: Inspiration) -> Result<usize> {
    match CORE.lock().await.as_ref() {
        None => Err(anyhow::anyhow!("Core not initialized")),
        Some(core) => core.sender_input(input),
    }
}
