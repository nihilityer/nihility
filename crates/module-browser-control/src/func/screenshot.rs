use crate::error::*;
use crate::BrowserControl;
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::page::ScreenshotParams;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

/// 截图网页
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScreenshotParam {
    /// 标签页对于的Id
    pub page_id: String,
    /// 需要被截图元素的`selector`
    pub selector: Option<String>,
}

impl BrowserControl {
    pub async fn screenshot(&self, param: ScreenshotParam) -> Result<Vec<u8>> {
        match self.page_map.get(&Uuid::from_str(&param.page_id)?) {
            None => Err(BrowserControlError::Operation(format!(
                "Invalid page id: {}",
                param.page_id
            ))),
            Some(page) => match param.selector {
                None => Ok(page
                    .screenshot(
                        ScreenshotParams::builder()
                            .format(CaptureScreenshotFormat::Png)
                            .full_page(true)
                            .omit_background(true)
                            .build(),
                    )
                    .await?),
                Some(selector) => Ok(page
                    .find_element(selector)
                    .await?
                    .screenshot(CaptureScreenshotFormat::Png)
                    .await?),
            },
        }
    }
}
