use crate::MODEL;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait NihilityModel: Send + Sync {
    async fn get_embedding(&self, text: &String) -> Result<Vec<f32>>;

    async fn get_chat_completion(&self, system: String, user: String) -> Result<Value>;
}

pub async fn get_embedding(text: &String) -> Result<Vec<f32>> {
    match MODEL.lock().await.as_ref() {
        None => Err(anyhow!("Model not initialized")),
        Some(model) => model.get_embedding(text).await,
    }
}

pub async fn get_chat_completion(system: String, user: String) -> Result<Value> {
    match MODEL.lock().await.as_ref() {
        None => Err(anyhow!("Model not initialized")),
        Some(model) => model.get_chat_completion(system, user).await,
    }
}
