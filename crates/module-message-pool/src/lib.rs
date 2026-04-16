pub mod analysis;
pub mod error;
pub mod func;

pub use analysis::{analysis_worker, GroupIdTask};
pub use error::MessagePoolError;
use std::sync::Arc;
use std::time::Duration;

use crate::error::*;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tokio::task::JoinHandle;
use tracing::{error, info};
use uuid::Uuid;

/// 分析器类型枚举
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema, Hash, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AnalyzerType {
    /// 命令分析器 - 检测以 `/` 开头的消息
    CommandAnalysis,
    /// 意图识别器 - 使用提示词进行意图识别
    IntentRecognition,
}

/// 分析器配置
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct AnalyzerConfig {
    /// 分析器类型
    pub analyzer_type: AnalyzerType,
    /// 是否启用
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// 优先级（数值越小越先执行）
    #[serde(default = "default_priority")]
    pub priority: i32,
}

fn default_enabled() -> bool {
    true
}

fn default_priority() -> i32 {
    0
}

/// 消息池模块配置
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
pub struct MessagePoolConfig {
    /// 分析器列表
    #[serde(default)]
    pub analyzers: Vec<AnalyzerConfig>,
}

impl Default for MessagePoolConfig {
    fn default() -> Self {
        Self {
            analyzers: vec![
                AnalyzerConfig {
                    analyzer_type: AnalyzerType::CommandAnalysis,
                    enabled: true,
                    priority: i32::MIN,
                },
                AnalyzerConfig {
                    analyzer_type: AnalyzerType::IntentRecognition,
                    enabled: true,
                    priority: 0,
                },
            ],
        }
    }
}

/// 消息内容枚举
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub enum ContentData {
    /// 文本消息
    Text {
        /// 文本内容
        body: String,
    },
    /// 音频消息
    Audio {
        /// 音频数据（URL 或 Base64）
        source: String,
    },
    /// 图片消息
    Image {
        /// 图片数据（URL 或 Base64）
        source: String,
    },
    /// 视频消息
    Video {
        /// 视频数据（URL 或 Base64）
        source: String,
    },
}

impl ContentData {
    /// 获取对应的 MsgType
    pub fn to_msg_type(&self) -> nihility_store_operate::message::MsgType {
        match self {
            ContentData::Text { .. } => nihility_store_operate::message::MsgType::Text,
            ContentData::Audio { .. } => nihility_store_operate::message::MsgType::Audio,
            ContentData::Image { .. } => nihility_store_operate::message::MsgType::Image,
            ContentData::Video { .. } => nihility_store_operate::message::MsgType::Video,
        }
    }
}

/// 消息结构体
/// content 枚举直接包含消息类型，可以区分文本/音频/图片/视频
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Message {
    /// 消息内容（包含类型信息）
    pub content: ContentData,
    /// 消息元数据（JSON Value，AI 引擎按需解析）
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// 场景信息
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SceneInfo {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub metadata: serde_json::Value,
    pub children_ids: Vec<String>,
}

/// 消息池主结构
pub struct MessagePool {
    conn: DatabaseConnection,
    task_tx: mpsc::UnboundedSender<Uuid>,
    analysis_task: Option<JoinHandle<Result<()>>>,
}

impl MessagePool {
    pub async fn init(config: MessagePoolConfig, conn: DatabaseConnection) -> Result<Self> {
        let (task_tx, task_rx) = mpsc::unbounded_channel::<Uuid>();

        info!("MessagePool initialized with config: {:?}", config);
        let task = tokio::spawn(analysis_worker(task_rx, config, conn.clone()));

        Ok(Self {
            conn,
            task_tx,
            analysis_task: Some(task),
        })
    }

    /// 触发分析链
    pub fn trigger_analysis(&self, group_id: Uuid) {
        if let Err(e) = self.task_tx.send(group_id) {
            tracing::warn!("Failed to send analysis task: {}", e);
        }
    }
}

pub async fn monitor_task(module: Arc<RwLock<MessagePool>>) {
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let mut module = module.write().await;
        if let Some(task) = module.analysis_task.as_ref()
            && task.is_finished()
            && let Some(task) = module.analysis_task.take()
        {
            match task.await {
                Ok(Ok(())) => info!("Analysis task finished"),
                Ok(Err(e)) => {
                    error!("Analysis task failed: {}", e);
                }
                Err(join_err) => {
                    error!("Analysis task join failed: {}", join_err);
                }
            }
        }
    }
}
