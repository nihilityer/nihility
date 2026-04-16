use crate::func::create_scene::CreateSceneParam;
use crate::func::delete_scene::DeleteSceneParam;
use crate::func::get_scene::GetSceneParam;
use crate::func::list_scenes::ListScenesParam;
use crate::func::update_scene::UpdateSceneParam;
use crate::Scene;
use nihility_module::{BoxStream, Callable, FunctionMetadata, Module};
use schemars::schema_for;
use serde_json::Value;
use tracing::debug;

pub mod create_scene;
pub mod delete_scene;
pub mod get_scene;
pub mod list_scenes;
pub mod update_scene;

#[async_trait::async_trait]
impl Callable for Scene {
    async fn call(&self, func_name: &str, param: Value) -> anyhow::Result<Value> {
        debug!(func_name = %func_name, param = ?param, "Scene module call");
        match func_name {
            "get_scene" => Ok(serde_json::to_value(
                self.get_scene(serde_json::from_value(param)?).await?,
            )?),
            "list_scenes" => Ok(serde_json::to_value(
                self.list_scenes(serde_json::from_value(param)?).await?,
            )?),
            _ => Err(anyhow::anyhow!("Unsupported func_name: {}", func_name)),
        }
    }

    async fn call_mut(&mut self, func_name: &str, param: Value) -> anyhow::Result<Value> {
        debug!(func_name = %func_name, param = ?param, "Scene module call_mut");
        match func_name {
            "create_scene" => Ok(serde_json::to_value(
                self.create_scene(serde_json::from_value(param)?).await?,
            )?),
            "update_scene" => Ok(serde_json::to_value(
                self.update_scene(serde_json::from_value(param)?).await?,
            )?),
            "delete_scene" => Ok(serde_json::to_value(
                self.delete_scene(serde_json::from_value(param)?).await?,
            )?),
            _ => Err(anyhow::anyhow!("Unsupported func_name: {}", func_name)),
        }
    }

    async fn call_stream(
        &self,
        func_name: &str,
        _param: Value,
    ) -> anyhow::Result<BoxStream<Value>> {
        debug!(func_name = %func_name, "Scene module does not support streaming");
        Err(anyhow::anyhow!("Scene does not support streaming"))
    }
}

impl Module for Scene {
    fn description(&self) -> &str {
        "场景管理模块，提供创建、查询、更新、删除场景等功能"
    }

    fn no_perm_func(&self) -> Vec<FunctionMetadata> {
        vec![
            FunctionMetadata {
                name: "get_scene".to_string(),
                desc: "获取单个场景信息".to_string(),
                tags: vec!["read".into()],
                params: serde_json::to_value(schema_for!(GetSceneParam))
                    .expect("scene func get_scene build param"),
            },
            FunctionMetadata {
                name: "list_scenes".to_string(),
                desc: "列出所有场景，支持按父场景筛选".to_string(),
                tags: vec!["read".into()],
                params: serde_json::to_value(schema_for!(ListScenesParam))
                    .expect("scene func list_scenes build param"),
            },
        ]
    }

    fn perm_func(&mut self) -> Vec<FunctionMetadata> {
        vec![
            FunctionMetadata {
                name: "create_scene".to_string(),
                desc: "创建新场景".to_string(),
                tags: vec!["write".into()],
                params: serde_json::to_value(schema_for!(CreateSceneParam))
                    .expect("scene func create_scene build param"),
            },
            FunctionMetadata {
                name: "update_scene".to_string(),
                desc: "更新场景信息".to_string(),
                tags: vec!["write".into()],
                params: serde_json::to_value(schema_for!(UpdateSceneParam))
                    .expect("scene func update_scene build param"),
            },
            FunctionMetadata {
                name: "delete_scene".to_string(),
                desc: "删除场景".to_string(),
                tags: vec!["write".into()],
                params: serde_json::to_value(schema_for!(DeleteSceneParam))
                    .expect("scene func delete_scene build param"),
            },
        ]
    }
}
