use crate::EdgeDeviceControl;
use nihility_module::{Callable, FunctionMetadata, Module};
use schemars::schema_for;
use serde_json::Value;
use tracing::debug;

mod connect_device;
mod list_devices;

use crate::func::connect_device::ConnectDeviceParam;
use crate::func::list_devices::ListDevicesParam;

#[async_trait::async_trait]
impl Callable for EdgeDeviceControl {
    async fn call(&self, func_name: &str, param: Value) -> anyhow::Result<Value> {
        debug!(
            func_name = %func_name,
            param = ?param,
            "EdgeDeviceControl call"
        );

        match func_name {
            "list_devices" => {
                let devices = self.list_devices().await;
                Ok(serde_json::to_value(devices)?)
            }
            _ => Err(anyhow::anyhow!("Unsupported func_name in call")),
        }
    }

    async fn call_mut(&mut self, func_name: &str, param: Value) -> anyhow::Result<Value> {
        debug!(
            func_name = %func_name,
            param = ?param,
            "EdgeDeviceControl call_mut"
        );

        match func_name {
            "connect_device" => Ok(serde_json::to_value(
                self.connect_device(serde_json::from_value(param)?).await?,
            )?),
            _ => Err(anyhow::anyhow!("Unsupported func_name in call_mut")),
        }
    }
}

impl Module for EdgeDeviceControl {
    fn no_perm_func(&self) -> Vec<FunctionMetadata> {
        vec![FunctionMetadata {
            name: "list_devices".to_string(),
            desc: "列出所有发现的边缘设备".to_string(),
            tags: vec!["edge".to_string(), "query".to_string()],
            params: serde_json::to_value(schema_for!(ListDevicesParam))
                .expect("edge control func list_devices build param"),
        }]
    }

    fn perm_func(&mut self) -> Vec<FunctionMetadata> {
        vec![FunctionMetadata {
            name: "connect_device".to_string(),
            desc: "连接指定的设备".to_string(),
            tags: vec!["edge".to_string(), "connect".to_string()],
            params: serde_json::to_value(schema_for!(ConnectDeviceParam))
                .expect("edge control func connect_device build param"),
        }]
    }
}
