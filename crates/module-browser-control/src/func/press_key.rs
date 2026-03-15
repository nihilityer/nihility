use crate::error::*;
use crate::BrowserControl;
use chromiumoxide::cdp::browser_protocol::input::{DispatchKeyEventParams, DispatchKeyEventType};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

/// 模拟按键输入
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PressKeyParam {
    /// 标签页对应的Id
    pub page_id: String,
    /// 按键Key
    pub key: String,
}

impl BrowserControl {
    pub async fn press_key(&mut self, param: PressKeyParam) -> Result<()> {
        match self.page_map.get(&Uuid::from_str(&param.page_id)?) {
            None => Err(BrowserControlError::Operation(format!(
                "Invalid page id: {}",
                param.page_id
            ))),
            Some(page) => {
                for type_ in [DispatchKeyEventType::KeyDown, DispatchKeyEventType::KeyUp] {
                    let params = DispatchKeyEventParams::builder()
                        .r#type(type_)
                        .key(&param.key)
                        .build()
                        .map_err(BrowserControlError::ExecuteParam)?;
                    page.execute(params).await?;
                }
                Ok(())
            }
        }
    }
}
