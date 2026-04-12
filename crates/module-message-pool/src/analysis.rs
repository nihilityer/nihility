mod command_analysis;
mod intent_recognition;

use crate::analysis::command_analysis::CommandAnalyzer;
use crate::analysis::intent_recognition::IntentAnalyzer;
use crate::{AnalyzerType, ContentData, MessageMetadata, MessagePoolConfig, MessagePoolError};
use async_trait::async_trait;
use nihility_store_operate::message::MsgType;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};
use uuid::Uuid;

/// 分析上下文
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    pub scene_id: Uuid,
    pub message_id: Uuid,
}

/// 分析任务消息
#[derive(Debug, Clone)]
pub struct AnalysisTask {
    pub scene_id: Uuid,
    pub message_id: Uuid,
}

/// 分析器特征
/// 返回 true 继续执行后续分析器，返回 false 终止分析链
#[async_trait]
pub trait Analyzer: Send + Sync {
    /// 分析器名称
    fn name(&self) -> &str;

    /// 优先级（数值越小越先执行）
    fn priority(&self) -> i32;

    /// 执行分析
    async fn analyze(
        &self,
        content: &ContentData,
        metadata: &MessageMetadata,
        context: &AnalysisContext,
    ) -> Result<bool, MessagePoolError>;
}

/// 分析引擎
#[allow(dead_code)]
pub struct AnalysisEngine {
    config: MessagePoolConfig,
    /// 已注册的分析器列表
    analyzers: Vec<Arc<dyn Analyzer>>,
    /// 分析任务发送通道
    task_tx: mpsc::Sender<AnalysisTask>,
}

impl AnalysisEngine {
    /// 创建分析引擎
    pub fn new(config: MessagePoolConfig, task_tx: mpsc::Sender<AnalysisTask>) -> Self {
        let analyzers = Self::create_analyzers(&config);
        Self {
            config,
            analyzers,
            task_tx,
        }
    }

    /// 根据配置创建分析器实例
    fn create_analyzers(config: &MessagePoolConfig) -> Vec<Arc<dyn Analyzer>> {
        let mut analyzers: Vec<Arc<dyn Analyzer>> = Vec::new();

        for analyzer_config in &config.analyzers {
            if !analyzer_config.enabled {
                continue;
            }

            let analyzer: Option<Arc<dyn Analyzer>> = match &analyzer_config.analyzer_type {
                AnalyzerType::CommandAnalysis => {
                    Some(Arc::new(CommandAnalyzer::new(analyzer_config.priority))
                        as Arc<dyn Analyzer>)
                }
                AnalyzerType::IntentRecognition => {
                    Some(Arc::new(IntentAnalyzer::new(analyzer_config.priority))
                        as Arc<dyn Analyzer>)
                }
            };

            if let Some(a) = analyzer {
                analyzers.push(a);
            }
        }

        // 按优先级排序（从小到大）
        analyzers.sort_by_key(|a| a.priority());

        info!("Analysis engine created with {} analyzers", analyzers.len());
        for a in &analyzers {
            info!("  - {} (priority: {})", a.name(), a.priority());
        }

        analyzers
    }

    /// 获取任务发送通道
    pub fn task_sender(&self) -> mpsc::Sender<AnalysisTask> {
        self.task_tx.clone()
    }

    /// 获取分析器列表（用于启动 worker）
    pub fn get_analyzers(&self) -> Vec<Arc<dyn Analyzer>> {
        self.analyzers.clone()
    }

    /// 启动消费者 worker，持续监听分析任务
    pub async fn start_worker(
        mut task_rx: mpsc::Receiver<AnalysisTask>,
        analyzers: Vec<Arc<dyn Analyzer>>,
        conn: sea_orm::DatabaseConnection,
    ) {
        info!("Analysis worker started");
        while let Some(task) = task_rx.recv().await {
            info!(
                "Received analysis task for message {} in scene {}",
                task.message_id, task.scene_id
            );

            // 从数据库加载消息内容
            match Self::load_message(&conn, task.message_id).await {
                Ok((_, content, metadata)) => {
                    let context = AnalysisContext {
                        scene_id: task.scene_id,
                        message_id: task.message_id,
                    };

                    // 执行分析链
                    for analyzer in &analyzers {
                        debug!("Running analyzer: {}", analyzer.name());

                        match analyzer.analyze(&content, &metadata, &context).await {
                            Ok(should_continue) => {
                                if !should_continue {
                                    info!(
                                        "Analyzer {} returned false, terminating analysis chain",
                                        analyzer.name()
                                    );
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("Analyzer {} failed: {}", analyzer.name(), e);
                            }
                        }
                    }

                    info!("Analysis chain completed for message {}", task.message_id);
                }
                Err(e) => {
                    error!(
                        "Failed to load message {} for analysis: {}",
                        task.message_id, e
                    );
                }
            }
        }
        info!("Analysis worker stopped");
    }

    /// 从数据库加载消息
    async fn load_message(
        conn: &sea_orm::DatabaseConnection,
        message_id: Uuid,
    ) -> Result<(MsgType, ContentData, MessageMetadata), MessagePoolError> {
        use nihility_store_operate::message;

        let message = message::find_message_by_id(conn, message_id).await?;

        let msg_type = message.msg_type;
        let content: ContentData = serde_json::from_value(message.content)?;
        let metadata: MessageMetadata = serde_json::from_value(message.metadata)?;

        Ok((msg_type, content, metadata))
    }
}
