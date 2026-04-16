pub mod config;
pub mod error;
pub mod func;
pub mod pool;
pub mod provider;
mod utils;

pub use config::ModelConfig;
use error::Result;
use pool::ModelPool;
use std::sync::Arc;
use tracing::info;

/// 模型模块主结构
pub struct Model {
    pool: Arc<ModelPool>,
}

impl Model {
    pub async fn init_from_file_config() -> Result<Self> {
        Self::init(nihility_config::get_config::<ModelConfig>(env!(
            "CARGO_PKG_NAME"
        ))?)
        .await
    }

    pub async fn init_from_db_config(conn: sea_orm::DatabaseConnection) -> Result<Self> {
        Self::init(
            nihility_config::get_config_with_db::<ModelConfig>(env!("CARGO_PKG_NAME"), &conn)
                .await?,
        )
        .await
    }

    pub async fn init(config: ModelConfig) -> Result<Self> {
        let pool = ModelPool::new(config);
        info!(
            "Model module initialized with {} models",
            pool.model_names().await.len()
        );
        Ok(Self {
            pool: Arc::new(pool),
        })
    }
}
