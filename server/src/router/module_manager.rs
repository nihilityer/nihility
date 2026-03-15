use crate::error::*;
use crate::AppState;
use axum::extract::{Path, State};
use axum::Json;
use nihility_module_manager::{EmbedModule, ModuleFunctions, ModuleType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// 查询所有模块的功能列表
pub async fn query_all_functions(
    State(state): State<AppState>,
) -> Result<Json<HashMap<ModuleType, ModuleFunctions>>> {
    let functions = state.module_manager.query_functions().await;
    Ok(Json(functions))
}

/// 查询指定模块的功能列表
pub async fn query_module_functions(
    State(state): State<AppState>,
    Path(module_type): Path<String>,
) -> Result<Json<ModuleFunctions>> {
    let module_type = parse_module_type(&module_type)?;
    let functions = state
        .module_manager
        .query_module_functions(&module_type)
        .await?;
    Ok(Json(functions))
}

/// 获取已加载的模块列表
pub async fn get_loaded_modules(State(state): State<AppState>) -> Result<Json<Vec<ModuleType>>> {
    let modules = state.module_manager.loaded_modules();
    Ok(Json(modules))
}

/// 模块方法调用请求
#[derive(Debug, Deserialize, Serialize)]
pub struct CallRequest {
    pub func_name: String,
    pub param: Value,
    pub is_mut: bool,
}

/// 模块方法调用响应
#[derive(Debug, Deserialize, Serialize)]
pub struct CallResponse {
    pub result: Value,
}

/// 调用指定模块的方法
pub async fn call_module_function(
    State(state): State<AppState>,
    Path(module_type): Path<String>,
    Json(request): Json<CallRequest>,
) -> Result<Json<CallResponse>> {
    let module_type = parse_module_type(&module_type)?;

    let result = if request.is_mut {
        state
            .module_manager
            .call_mut(&module_type, &request.func_name, request.param)
            .await?
    } else {
        state
            .module_manager
            .call(&module_type, &request.func_name, request.param)
            .await?
    };

    Ok(Json(CallResponse { result }))
}

/// 解析模块类型字符串
fn parse_module_type(module_type_str: &str) -> Result<ModuleType> {
    match module_type_str {
        "browser_control" => Ok(ModuleType::Embed(EmbedModule::BrowserControl)),
        "edge_device_control" => Ok(ModuleType::Embed(EmbedModule::EdgeDeviceControl)),
        _ => Err(NihilityServerError::NotFound(format!(
            "Module type: {}",
            module_type_str
        ))),
    }
}
