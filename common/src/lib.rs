pub mod inspiration;
pub mod idea;

use crate::inspiration::Inspiration;
use crate::idea::Idea;
use anyhow::Result;
use lazy_static::lazy_static;
use tokio::sync::broadcast::Receiver;
use tokio::sync::{Mutex, broadcast, mpsc};
use tracing::error;
use uuid::Uuid;

lazy_static! {
    static ref GLOBAL_SENDER: Mutex<Option<mpsc::Sender<Inspiration>>> = Mutex::new(None);
    static ref CHAT_OUTPUT: Mutex<broadcast::Sender<Idea >> = {
        let (tx, _) = broadcast::channel(10);
        Mutex::new(tx)
    };
    static ref MEMORY_IDEA: Mutex<broadcast::Sender<Idea >> = {
        let (tx, _) = broadcast::channel(10);
        Mutex::new(tx)
    };
    static ref THINK: Mutex<Option<String>> = Mutex::new(None);
}

/// 设置当前思考
pub async fn set_think<T: Into<String>>(think: T) {
    THINK.lock().await.replace(think.into());
}

/// 获取当前思考
pub async fn get_think() -> Option<String> {
    THINK.lock().await.clone()
}

/// 这个方法必须在其他信息输入组件注册之前运行
pub async fn init_inspiration_sender(buffer: usize) -> mpsc::Receiver<Inspiration> {
    let (tx, rx) = mpsc::channel(buffer);
    *GLOBAL_SENDER.lock().await = Some(tx);
    rx
}

/// 注册一个信息输入组件
pub async fn register_inspiration_plugin(mut receiver: Receiver<Inspiration>) -> Result<Uuid> {
    match GLOBAL_SENDER.lock().await.clone() {
        None => Err(anyhow::anyhow!("Global sender not initialized")),
        Some(sender) => {
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
    }
}

/// 注册聊天输出组件,所有注册的组件都会受到消息
pub async fn register_chat_output_plugin() -> broadcast::Receiver<Idea> {
    CHAT_OUTPUT.lock().await.subscribe()
}

/// 发送聊天输出
pub async fn sender_chat_output(output: Idea) -> Result<usize> {
    Ok(CHAT_OUTPUT.lock().await.send(output)?)
}

pub async fn register_memory_idea_plugin() -> broadcast::Receiver<Idea> {
    MEMORY_IDEA.lock().await.subscribe()
}

pub async fn sender_memory_idea(idea: Idea) -> Result<usize> {
    Ok(MEMORY_IDEA.lock().await.send(idea)?)
}
