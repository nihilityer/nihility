pub mod error;

use crate::error::*;
use nihility_module::{BoxStream, FunctionMetadata, Module};
use nihility_module_edge_device_control::EdgeDeviceControl;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::error;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum ModuleType {
    Embed(EmbedModule),
    Wasm(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum EmbedModule {
    BrowserControl,
    EdgeDeviceControl,
    Model,
    MessagePool,
}

/// 模块功能列表
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModuleFunctions {
    /// 模块描述
    pub description: String,
    /// 低权限功能列表
    pub no_perm_func: Vec<FunctionMetadata>,
    /// 高权限功能列表
    pub perm_func: Vec<FunctionMetadata>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModuleManagerConfig {
    pub enable_modules: Vec<ModuleType>,
}

pub struct ModuleManager {
    modules: HashMap<ModuleType, Arc<RwLock<dyn Module + Send + Sync>>>,
    edge_device_control: Option<Arc<RwLock<EdgeDeviceControl>>>,
}

impl ModuleManager {
    pub async fn init_from_file_config(conn: DatabaseConnection) -> Result<Self> {
        Self::init(
            nihility_config::get_config::<ModuleManagerConfig>(env!("CARGO_PKG_NAME"))?,
            conn,
        )
        .await
    }

    pub async fn init(mut config: ModuleManagerConfig, conn: DatabaseConnection) -> Result<Self> {
        config = config.sorted();
        let mut modules: HashMap<ModuleType, Arc<RwLock<dyn Module + Send + Sync>>> =
            HashMap::new();

        let mut browser_control = None;
        let mut edge_device_control = None;
        let mut model = None;

        for enable_module in config.enable_modules {
            match enable_module {
                ModuleType::Embed(embed_module) => match embed_module {
                    EmbedModule::BrowserControl => {
                        let config =
                            nihility_config::get_config_with_db::<
                                nihility_module_browser_control::BrowserControlConfig,
                            >("nihility-module-browser-control", &conn)
                            .await?;
                        let module = Arc::new(RwLock::new(
                            nihility_module_browser_control::BrowserControl::init(config).await?,
                        ));
                        browser_control = Some(module.clone());
                        modules.insert(ModuleType::Embed(embed_module), module);
                    }
                    EmbedModule::Model => {
                        let config = nihility_config::get_config_with_db::<
                            nihility_module_model::ModelConfig,
                        >("nihility-module-model", &conn)
                        .await?;
                        let module = Arc::new(RwLock::new(
                            nihility_module_model::Model::init(config).await?,
                        ));
                        model = Some(module.clone());
                        modules.insert(ModuleType::Embed(embed_module), module);
                    }
                    EmbedModule::EdgeDeviceControl => {
                        let config = nihility_config::get_config_with_db::<
                            nihility_module_edge_device_control::EdgeDeviceControlConfig,
                        >(
                            "nihility-module-edge-device-control", &conn
                        )
                        .await?;
                        let mut module = EdgeDeviceControl::init(config).await?;
                        if let Some(browser_control) = browser_control.as_ref() {
                            module.set_browser_control(browser_control.clone());
                        } else {
                            error!(
                                "browser_control module does not exist for module type: {:?}",
                                embed_module
                            );
                        }
                        if let Some(model) = model.as_ref() {
                            module.set_model(model.clone()).await?;
                        } else {
                            error!(
                                "model module does not exist for module type: {:?}",
                                embed_module
                            );
                        }
                        let module = Arc::new(RwLock::new(module));
                        let monitor_module = module.clone();
                        tokio::spawn(nihility_module_edge_device_control::monitor_task(
                            monitor_module,
                        ));
                        edge_device_control = Some(module.clone());
                        modules.insert(ModuleType::Embed(embed_module), module);
                    }
                    EmbedModule::MessagePool => {
                        let config =
                            nihility_config::get_config_with_db::<
                                nihility_module_message_pool::MessagePoolConfig,
                            >("nihility-module-message-pool", &conn)
                            .await?;
                        let module = Arc::new(RwLock::new(
                            nihility_module_message_pool::MessagePool::init(config, conn.clone())
                                .await?,
                        ));
                        modules.insert(ModuleType::Embed(embed_module), module);
                    }
                },
                ModuleType::Wasm(path) => {
                    error!("wasm module not support yet: {}", path);
                }
            }
        }
        Ok(Self {
            modules,
            edge_device_control,
        })
    }

    pub fn get_edge_device_control(&self) -> Result<Arc<RwLock<EdgeDeviceControl>>> {
        self.edge_device_control
            .as_ref()
            .map(Clone::clone)
            .ok_or_else(|| {
                ModuleManagerError::ModuleNotFound(ModuleType::Embed(
                    EmbedModule::EdgeDeviceControl,
                ))
            })
    }

    /// 查询所有模块的功能列表
    /// 返回: HashMap<ModuleType, ModuleFunctions>
    pub async fn query_functions(&self) -> HashMap<ModuleType, ModuleFunctions> {
        let mut result = HashMap::new();

        for (module_type, module) in &self.modules {
            let mut module_guard = module.write().await;
            let functions = ModuleFunctions {
                description: module_guard.description().to_string(),
                no_perm_func: module_guard.no_perm_func(),
                perm_func: module_guard.perm_func(),
            };
            result.insert(module_type.clone(), functions);
        }

        result
    }

    /// 查询指定模块的功能列表
    pub async fn query_module_functions(
        &self,
        module_type: &ModuleType,
    ) -> Result<ModuleFunctions> {
        let module = self
            .modules
            .get(module_type)
            .ok_or_else(|| ModuleManagerError::ModuleNotFound(module_type.clone()))?;

        let mut module_guard = module.write().await;
        Ok(ModuleFunctions {
            description: module_guard.description().to_string(),
            no_perm_func: module_guard.no_perm_func(),
            perm_func: module_guard.perm_func(),
        })
    }

    /// 调用指定模块的指定方法（不可变调用）
    pub async fn call(
        &self,
        module_type: &ModuleType,
        func_name: &str,
        param: Value,
    ) -> Result<Value> {
        let module = self
            .modules
            .get(module_type)
            .ok_or_else(|| ModuleManagerError::ModuleNotFound(module_type.clone()))?;

        let module_guard = module.read().await;
        module_guard
            .call(func_name, param)
            .await
            .map_err(ModuleManagerError::Anyhow)
    }

    /// 调用指定模块的指定方法（可变调用）
    pub async fn call_mut(
        &self,
        module_type: &ModuleType,
        func_name: &str,
        param: Value,
    ) -> Result<Value> {
        let module = self
            .modules
            .get(module_type)
            .ok_or_else(|| ModuleManagerError::ModuleNotFound(module_type.clone()))?;

        let mut module_guard = module.write().await;
        module_guard
            .call_mut(func_name, param)
            .await
            .map_err(ModuleManagerError::Anyhow)
    }

    /// 流式调用指定模块的指定方法
    pub async fn stream_call(
        &self,
        module_type: &ModuleType,
        func_name: &str,
        param: Value,
    ) -> Result<BoxStream<Value>> {
        let module = self
            .modules
            .get(module_type)
            .ok_or_else(|| ModuleManagerError::ModuleNotFound(module_type.clone()))?;

        let module_guard = module.read().await;
        module_guard
            .call_stream(func_name, param)
            .await
            .map_err(ModuleManagerError::Anyhow)
    }

    /// 获取所有已加载的模块类型
    pub fn loaded_modules(&self) -> Vec<ModuleType> {
        self.modules.keys().cloned().collect()
    }
}

impl Default for ModuleManagerConfig {
    fn default() -> Self {
        Self {
            enable_modules: vec![
                ModuleType::Embed(EmbedModule::BrowserControl),
                ModuleType::Embed(EmbedModule::MessagePool),
                ModuleType::Embed(EmbedModule::EdgeDeviceControl),
                ModuleType::Embed(EmbedModule::Model),
            ],
        }
    }
}

impl ModuleManagerConfig {
    pub(crate) fn sorted(self) -> Self {
        let mut embeds: Vec<EmbedModule> = Vec::new();
        let mut wasms: Vec<String> = Vec::new();

        for module in self.enable_modules {
            match module {
                ModuleType::Embed(embed) => embeds.push(embed),
                ModuleType::Wasm(name) => wasms.push(name),
            }
        }

        embeds.sort_by_key(|embed| match embed {
            EmbedModule::BrowserControl => 0,
            EmbedModule::MessagePool => 1,
            EmbedModule::Model => 2,
            EmbedModule::EdgeDeviceControl => 3,
        });

        let mut enable_modules = Vec::with_capacity(embeds.len() + wasms.len());
        enable_modules.extend(embeds.into_iter().map(ModuleType::Embed));
        enable_modules.extend(wasms.into_iter().map(ModuleType::Wasm));

        ModuleManagerConfig { enable_modules }
    }
}

impl Serialize for EmbedModule {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            EmbedModule::BrowserControl => "browser-control",
            EmbedModule::EdgeDeviceControl => "edge-device-control",
            EmbedModule::Model => "model",
            EmbedModule::MessagePool => "message-pool",
        };
        serializer.serialize_str(s)
    }
}

impl<'de> Deserialize<'de> for EmbedModule {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "browser-control" => Ok(EmbedModule::BrowserControl),
            "edge-device-control" => Ok(EmbedModule::EdgeDeviceControl),
            "model" => Ok(EmbedModule::Model),
            "message-pool" => Ok(EmbedModule::MessagePool),
            _ => Err(serde::de::Error::custom(format!(
                "unknown embed module: {}",
                s
            ))),
        }
    }
}

impl Serialize for ModuleType {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = match self {
            ModuleType::Embed(embed) => {
                let embed_str = match embed {
                    EmbedModule::BrowserControl => "browser-control",
                    EmbedModule::EdgeDeviceControl => "edge-device-control",
                    EmbedModule::Model => "model",
                    EmbedModule::MessagePool => "message-pool",
                };
                format!("embed-{}", embed_str)
            }
            ModuleType::Wasm(path) => format!("wasm-{}", path),
        };
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for ModuleType {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        if let Some(embed_name) = s.strip_prefix("embed-") {
            match embed_name {
                "browser-control" => Ok(ModuleType::Embed(EmbedModule::BrowserControl)),
                "edge-device-control" => Ok(ModuleType::Embed(EmbedModule::EdgeDeviceControl)),
                "model" => Ok(ModuleType::Embed(EmbedModule::Model)),
                "message-pool" => Ok(ModuleType::Embed(EmbedModule::MessagePool)),
                _ => Err(serde::de::Error::custom(format!(
                    "unknown embed module: {}",
                    embed_name
                ))),
            }
        } else if let Some(wasm_path) = s.strip_prefix("wasm-") {
            Ok(ModuleType::Wasm(wasm_path.to_string()))
        } else {
            Err(serde::de::Error::custom(format!(
                "invalid module type format: {}, expected 'embed-{{module}}' or 'wasm-{{path}}'",
                s
            )))
        }
    }
}
