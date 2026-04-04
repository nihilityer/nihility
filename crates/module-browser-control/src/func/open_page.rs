use crate::error::*;
use crate::BrowserControl;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 打开新标签页
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenPageParam {
    /// 网页地址
    pub url: String,
}

impl BrowserControl {
    pub async fn open_page(&mut self, param: OpenPageParam) -> Result<Uuid> {
        let page = self.browser.new_page(param.url).await?;
        let page_id = Uuid::new_v4();
        self.page_map.insert(page_id, page);
        Ok(page_id)
    }
}
