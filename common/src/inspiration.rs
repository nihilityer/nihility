use serde::{Deserialize, Serialize};

/// 灵感
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Inspiration {
    ChatApp(String),
    Memory(String),
}
