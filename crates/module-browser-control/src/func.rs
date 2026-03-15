use crate::func::open_page::OpenPageParam;
use crate::func::press_key::PressKeyParam;
use crate::func::screenshot::ScreenshotParam;
use crate::BrowserControl;
use nihility_module::{Callable, FunctionMetadata, Module};
use schemars::schema_for;
use serde_json::Value;
use tracing::debug;

pub mod open_page;
pub mod press_key;
pub mod screenshot;

#[async_trait::async_trait]
impl Callable for BrowserControl {
    async fn call(&self, func_name: &str, param: Value) -> anyhow::Result<Value> {
        debug!(
            func_name = %func_name,
            param = ?param,
            "Browser control call"
        );
        match func_name {
            "screenshot" => Ok(serde_json::to_value(
                self.screenshot(serde_json::from_value(param)?).await?,
            )?),
            _ => Err(anyhow::anyhow!("Unsupported func_name")),
        }
    }

    async fn call_mut(&mut self, func_name: &str, param: Value) -> anyhow::Result<Value> {
        debug!(
            func_name = %func_name,
            param = ?param,
            "Browser control call_mut"
        );
        match func_name {
            "open_page" => Ok(serde_json::to_value(
                self.open_page(serde_json::from_value(param)?).await?,
            )?),
            "press_key" => Ok(serde_json::to_value(
                self.press_key(serde_json::from_value(param)?).await?,
            )?),
            _ => Err(anyhow::anyhow!("Unsupported func_name")),
        }
    }
}

impl Module for BrowserControl {
    fn description(&self) -> &str {
        "浏览器控制模块，提供网页打开、截图、按键输入等浏览器自动化功能"
    }

    fn no_perm_func(&self) -> Vec<FunctionMetadata> {
        vec![FunctionMetadata {
            name: "screenshot".to_string(),
            desc: "截图网页".to_string(),
            tags: vec![],
            params: serde_json::to_value(schema_for!(ScreenshotParam))
                .expect("browser control func screenshot build param"),
        }]
    }

    fn perm_func(&mut self) -> Vec<FunctionMetadata> {
        vec![
            FunctionMetadata {
                name: "open_page".to_string(),
                desc: "打开网页".to_string(),
                tags: vec![],
                params: serde_json::to_value(schema_for!(OpenPageParam))
                    .expect("browser control func open_page build param"),
            },
            FunctionMetadata {
                name: "press_key".to_string(),
                desc: "模拟按键输入".to_string(),
                tags: vec![],
                params: serde_json::to_value(schema_for!(PressKeyParam))
                    .expect("browser control func press_key build param"),
            },
        ]
    }
}
