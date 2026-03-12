use crate::func::open_page::OpenPageParam;
use crate::func::screenshot::ScreenshotParam;
use crate::BrowserControl;
use nihility_module::{Callable, FunctionMetadata, Module};
use schemars::schema_for;
use serde_json::Value;
use tracing::debug;

pub mod open_page;
pub mod press_key;
pub mod screenshot;

#[async_trait::async_trait(?Send)]
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
            _ => Err(anyhow::anyhow!("Unsupported func_name")),
        }
    }
}

impl Module for BrowserControl {
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
        vec![FunctionMetadata {
            name: "open_page".to_string(),
            desc: "打开网页".to_string(),
            tags: vec![],
            params: serde_json::to_value(schema_for!(OpenPageParam))
                .expect("browser control func open_page build param"),
        }]
    }
}
