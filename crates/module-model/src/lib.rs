pub mod config;
pub mod error;
pub mod func;
pub mod pool;
pub mod provider;

use config::ModelConfig;
use error::Result;
use pool::ModelPool;
use std::sync::Arc;
use tracing::info;

/// 模型模块主结构
pub struct ModelModule {
    pool: Arc<ModelPool>,
}

impl ModelModule {
    /// 从配置文件初始化
    pub async fn init_from_file_config() -> Result<Self> {
        Self::init(nihility_config::get_config::<ModelConfig>(env!(
            "CARGO_PKG_NAME"
        ))?)
        .await
    }

    /// 直接初始化
    pub async fn init(config: ModelConfig) -> Result<Self> {
        let pool = ModelPool::new(&config);
        info!(
            "Model module initialized with {} models",
            pool.model_names().await.len()
        );
        Ok(Self {
            pool: Arc::new(pool),
        })
    }
}
