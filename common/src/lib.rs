pub mod context;
pub mod input;
pub mod output;

use crate::input::{ InputEntity};
use anyhow::Result;
use lazy_static::lazy_static;
use tokio::sync::{Mutex, mpsc, broadcast};
use tokio::sync::broadcast::Receiver;
use tracing::error;
use uuid::Uuid;
use crate::output::OutputEntity;

lazy_static! {
    static ref GLOBAL_SENDER: Mutex<Option<mpsc::Sender<InputEntity>>> = Mutex::new(None);
    static ref CHAT_OUTPUT: Mutex<broadcast::Sender<OutputEntity>> = {
        let (tx, _) = broadcast::channel(10);
        Mutex::new(tx)
    };
}

/// 这个方法必须在其他信息输入组件注册之前运行
pub async fn init_input_sender(buffer: usize) -> mpsc::Receiver<InputEntity> {
    let (tx, rx) = mpsc::channel(buffer);
    *GLOBAL_SENDER.lock().await = Some(tx);
    rx
}

/// 注册一个信息输入组件
pub async fn register_input_plugin(plugin: &dyn AssistantInput) -> Result<Uuid> {
    match GLOBAL_SENDER.lock().await.clone() {
        None => Err(anyhow::anyhow!("Global sender not initialized")),
        Some(sender) => match plugin.subscribe() {
            Ok(mut receiver) => {
                let uuid = Uuid::new_v4();
                let plugin_name = uuid;
                tokio::spawn(async move {
                    while let Ok(input) = receiver.recv().await {
                        if let Err(e) = sender.send(input).await {
                            error!("{} Error sending input: {:?}", plugin_name, e);
                        }
                    }
                });
                Ok(uuid)
            }
            Err(e) => Err(anyhow::anyhow!("Get subscribe error: {}", e)),
        },
    }
}

/// 注册聊天输出组件,所有注册的组件都会受到消息
pub async fn register_chat_output_plugin() -> broadcast::Receiver<OutputEntity> {
    CHAT_OUTPUT.lock().await.subscribe()
}

/// 发送聊天输出
pub async fn sender_chat_output(output: OutputEntity) -> Result<usize> {
    Ok(CHAT_OUTPUT.lock().await.send(output)?)
}

/// 信息输入组件
pub trait AssistantInput {
    /// 获取消息输入订阅
    fn subscribe(&self) -> Result<Receiver<InputEntity>>;
}
