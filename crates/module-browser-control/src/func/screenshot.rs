use crate::error::*;
use crate::BrowserControl;
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ScreenshotParam {
    pub url: String,
}

impl BrowserControl {
    pub(crate) async fn screenshot(&mut self, param: ScreenshotParam) -> Result<Vec<u8>> {
        let ele = self
            .browser
            .new_page(param.url)
            .await?
            .find_element("#app > div")
            .await?;

        Ok(ele.screenshot(CaptureScreenshotFormat::Png).await?)
    }
}
