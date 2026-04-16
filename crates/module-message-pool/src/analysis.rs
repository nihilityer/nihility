mod command_analysis;
mod intent_recognition;

use crate::analysis::command_analysis::CommandAnalyzer;
use crate::analysis::intent_recognition::IntentAnalyzer;
use crate::error::*;
use crate::{AnalyzerType, MessagePoolConfig};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info};
use uuid::Uuid;

/// 分组分析任务（只包含 group_id）
#[derive(Debug, Clone)]
pub struct GroupIdTask {
    pub group_id: Uuid,
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
    async fn analyze(&self, db: &DatabaseConnection, group_id: Uuid) -> Result<bool>;
}

/// 根据配置创建分析器
fn create_analyzer(config: &crate::AnalyzerConfig) -> Option<Arc<dyn Analyzer>> {
    match &config.analyzer_type {
        AnalyzerType::CommandAnalysis => {
            Some(Arc::new(CommandAnalyzer::new(config.priority)) as Arc<dyn Analyzer>)
        }
        AnalyzerType::IntentRecognition => {
            Some(Arc::new(IntentAnalyzer::new(config.priority)) as Arc<dyn Analyzer>)
        }
    }
}

pub async fn analysis_worker(
    mut task_rx: mpsc::UnboundedReceiver<Uuid>,
    config: MessagePoolConfig,
    conn: DatabaseConnection,
) -> Result<()> {
    // 根据配置创建分析器
    let mut analyzers: Vec<Arc<dyn Analyzer>> = Vec::new();

    for analyzer_config in &config.analyzers {
        if !analyzer_config.enabled {
            continue;
        }
        if let Some(analyzer) = create_analyzer(analyzer_config) {
            analyzers.push(analyzer);
        }
    }

    // 按优先级排序
    analyzers.sort_by_key(|a| a.priority());

    info!("Analysis worker started with {} analyzers", analyzers.len());
    for a in &analyzers {
        info!("  - {} (priority: {})", a.name(), a.priority(),);
    }

    while let Some(group_id) = task_rx.recv().await {
        info!("Received analysis task for group_id {}", group_id);

        // 按优先级顺序执行分析链
        for analyzer in &analyzers {
            debug!("Running analyzer: {}", analyzer.name());

            match analyzer.analyze(&conn, group_id).await {
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

        info!("Analysis chain completed for group_id {}", group_id);
    }

    info!("Analysis worker stopped");
    Ok(())
}
