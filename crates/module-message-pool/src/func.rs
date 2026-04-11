pub mod add_message;
pub mod get_scene_info;
pub mod process_messages;

use crate::MessagePool;
use nihility_module::{BoxStream, Callable, FunctionMetadata, Module};
use schemars::schema_for;
use serde_json::Value;
use tracing::debug;

#[async_trait::async_trait]
impl Callable for MessagePool {
    async fn call(&self, func_name: &str, param: Value) -> anyhow::Result<Value> {
        debug!(func_name = %func_name, param = ?param, "MessagePool call");
        match func_name {
            "add_messages" => Ok(serde_json::to_value(
                self.add_messages(serde_json::from_value(param)?).await?,
            )?),
            "get_scene_info" => Ok(serde_json::to_value(
                self.get_scene_info(serde_json::from_value(param)?).await?,
            )?),
            _ => Err(anyhow::anyhow!("Unsupported func_name: {}", func_name)),
        }
    }

    async fn call_mut(&mut self, func_name: &str, param: Value) -> anyhow::Result<Value> {
        debug!(func_name = %func_name, param = ?param, "MessagePool call_mut");
        match func_name {
            "process_scene_messages" => Ok(serde_json::to_value(
                self.process_scene_messages(serde_json::from_value(param)?).await?,
            )?),
            _ => Err(anyhow::anyhow!("Unsupported func_name: {}", func_name)),
        }
    }

    async fn call_stream(
        &self,
        func_name: &str,
        _param: Value,
    ) -> anyhow::Result<BoxStream<Value>> {
        debug!(func_name = %func_name, "MessagePool does not support streaming");
        Err(anyhow::anyhow!("MessagePool does not support streaming"))
    }
}

impl Module for MessagePool {
    fn description(&self) -> &str {
        "消息池模块，管理场景消息的存储、分析和处理"
    }

    fn no_perm_func(&self) -> Vec<FunctionMetadata> {
        vec![
            FunctionMetadata {
                name: "add_messages".to_string(),
                desc: "向消息池添加消息列表".to_string(),
                tags: vec![],
                params: serde_json::to_value(schema_for!(add_message::AddMessagesParam))
                    .expect("message pool func add_messages build param"),
            },
            FunctionMetadata {
                name: "get_scene_info".to_string(),
                desc: "获取场景信息".to_string(),
                tags: vec![],
                params: serde_json::to_value(schema_for!(get_scene_info::GetSceneInfoParam))
                    .expect("message pool func get_scene_info build param"),
            },
        ]
    }

    fn perm_func(&mut self) -> Vec<FunctionMetadata> {
        vec![FunctionMetadata {
            name: "process_scene_messages".to_string(),
            desc: "处理场景中所有未处理的消息".to_string(),
            tags: vec![],
            params: serde_json::to_value(schema_for!(process_messages::ProcessSceneMessagesParam))
                .expect("message pool func process_scene_messages build param"),
        }]
    }
}