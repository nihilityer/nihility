pub mod analysis;
pub mod error;
pub mod func;

pub use analysis::{AnalysisEngine, AnalysisTask};
pub use error::MessagePoolError;

use crate::error::*;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::info;
use uuid::Uuid;

/// 分析器类型枚举
#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
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
#[serde(tag = "type", content = "data")]
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
        /// 时长（秒）
        duration: Option<f64>,
        /// 采样率
        sample_rate: Option<u32>,
    },
    /// 图片消息
    Image {
        /// 图片数据（URL 或 Base64）
        source: String,
        /// 宽度
        width: Option<u32>,
        /// 高度
        height: Option<u32>,
    },
    /// 视频消息
    Video {
        /// 视频数据（URL 或 Base64）
        source: String,
        /// 时长（秒）
        duration: Option<f64>,
        /// 宽度
        width: Option<u32>,
        /// 高度
        height: Option<u32>,
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

/// 消息元数据
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema, Default)]
pub struct MessageMetadata {
    /// 来源设备 ID
    pub device_id: Option<String>,
    /// 原始格式
    pub format: Option<String>,
    /// 用户自定义标签
    #[serde(default)]
    pub tags: Vec<String>,
}

/// 消息结构体
/// content 枚举直接包含消息类型，可以区分文本/音频/图片/视频
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Message {
    /// 消息内容（包含类型信息）
    pub content: ContentData,
    /// 消息元数据
    #[serde(default)]
    pub metadata: MessageMetadata,
}

/// 场景信息
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct SceneInfo {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub metadata: serde_json::Value,
    /// 直接子场景 ID 列表
    pub children_ids: Vec<String>,
}

/// 消息池主结构
pub struct MessagePool {
    conn: DatabaseConnection,
    task_tx: mpsc::Sender<AnalysisTask>,
}

impl MessagePool {
    /// 从配置文件初始化
    pub async fn init_from_file_config(conn: DatabaseConnection) -> Result<Self> {
        let config = nihility_config::get_config::<MessagePoolConfig>(env!("CARGO_PKG_NAME"))?;
        Self::init(config, conn).await
    }

    /// 直接初始化
    pub async fn init(config: MessagePoolConfig, conn: DatabaseConnection) -> Result<Self> {
        // 创建分析任务通道
        let (task_tx, task_rx) = mpsc::channel::<AnalysisTask>(100);

        // 创建分析引擎并获取分析器列表
        let engine = AnalysisEngine::new(config.clone(), task_tx.clone());
        let analyzers = engine.get_analyzers();

        // 启动分析 worker
        let worker_conn = conn.clone();
        tokio::spawn(async move {
            AnalysisEngine::start_worker(task_rx, analyzers, worker_conn).await;
        });

        info!("MessagePool initialized with config: {:?}", config);
        Ok(Self { conn, task_tx })
    }

    /// 触发分析链
    pub fn trigger_analysis(&self, scene_id: Uuid, message_id: Uuid) {
        let task = AnalysisTask {
            scene_id,
            message_id,
        };

        // 发送任务到分析通道
        if let Err(e) = self.task_tx.try_send(task) {
            tracing::warn!("Failed to send analysis task: {}", e);
        }
    }
}
