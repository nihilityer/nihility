use serde::{Deserialize, Serialize};

/// 灵感
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Inspiration {
    External(String),
    Internal(String),
}
