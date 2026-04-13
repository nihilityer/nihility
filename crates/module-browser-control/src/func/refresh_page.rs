use crate::error::*;
use crate::BrowserControl;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 刷新网页
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RefreshPageParam {
    /// 标签页Id
    pub page_id: Uuid,
}

impl BrowserControl {
    pub async fn refresh_page(&mut self, param: RefreshPageParam) -> Result<()> {
        if let Some(page) = self.page_map.get(&param.page_id) {
            page.reload().await?;
        }
        Ok(())
    }
}