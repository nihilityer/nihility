pub mod config;
pub mod intention;
pub mod inspiration;
pub mod model;

use crate::config::NihilityConfig;
use crate::intention::Intention;
use crate::inspiration::Inspiration;
use crate::model::NihilityModel;
use anyhow::Result;
use lazy_static::lazy_static;
use tokio::sync::broadcast::Receiver;
use tokio::sync::{Mutex, broadcast, mpsc};
use tracing::error;

lazy_static! {
    static ref INSPIRATION_SENDER: Mutex<Option<mpsc::Sender<Inspiration>>> = Mutex::new(None);
    static ref INTENTION_SENDER: Mutex<broadcast::Sender<Intention >> = {
        let (tx, _) = broadcast::channel(10);
        Mutex::new(tx)
    };
    static ref MODEL: Mutex<Option<Box<dyn NihilityModel + Send + Sync>>> = Mutex::new(None);
    static ref CONFIG: Mutex<Option<Box<dyn NihilityConfig + Send + Sync>>> = Mutex::new(None);
}

pub async fn set_model(model: Box<dyn NihilityModel + Send + Sync>) {
    MODEL.lock().await.replace(model);
}

/// 这个方法必须在其他信息输入组件注册之前运行
pub async fn init_inspiration_sender(buffer: usize) -> mpsc::Receiver<Inspiration> {
    let (tx, rx) = mpsc::channel(buffer);
    *INSPIRATION_SENDER.lock().await = Some(tx);
    rx
}

/// 注册一个灵感输入组件
pub async fn register_inspiration_plugin(mut receiver: Receiver<Inspiration>) -> Result<()> {
    match INSPIRATION_SENDER.lock().await.clone() {
        None => Err(anyhow::anyhow!("Inspiration sender not initialized")),
        Some(sender) => {
            tokio::spawn(async move {
                while let Ok(input) = receiver.recv().await {
                    if let Err(e) = sender.send(input).await {
                        error!("Send inspiration to core error: {:?}", e);
                    }
                }
            });
            Ok(())
        }
    }
}

pub async fn register_intention_receiver_plugin() -> broadcast::Receiver<Intention> {
    INTENTION_SENDER.lock().await.subscribe()
}

pub async fn sender_intention(intention: Intention) -> Result<usize> {
    Ok(INTENTION_SENDER.lock().await.send(intention)?)
}
