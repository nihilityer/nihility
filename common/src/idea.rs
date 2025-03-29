use serde::{Deserialize, Serialize};

/// 信息输出实体
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Idea {
    ChatApp(String),
    Memory(String),
}
