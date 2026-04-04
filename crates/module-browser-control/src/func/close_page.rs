use crate::error::*;
use crate::BrowserControl;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 关闭标签页
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClosePageParam {
    /// 标签页Id
    pub page_id: Uuid,
}

impl BrowserControl {
    pub async fn close_page(&mut self, param: ClosePageParam) -> Result<()> {
        if let Some(page) = self.page_map.remove(&param.page_id) {
            page.close().await?;
        }
        Ok(())
    }
}
