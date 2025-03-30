use serde::{Deserialize, Serialize};

/// 想法
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intention {
    pub decision: DecisionType,
    pub detail: String,
}

/// 决策类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DecisionType {
    Nothing,
    Recall,
    Memorize,
    Explore,
    Express,
    Execute
}