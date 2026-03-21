use crate::error::*;
use crate::router::not_found;
use crate::AppState;
use axum::extract::{Path, State};
use axum::response::sse::{Event, Sse};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures::StreamExt;
use nihility_module_manager::{ModuleFunctions, ModuleType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::convert::Infallible;
use std::pin::Pin;

pub fn module_manager_router() -> Router<AppState> {
    Router::new()
        .route("/", get(get_loaded_modules))
        .route("/functions", get(query_all_functions))
        .route("/{module_type}/functions", get(query_module_functions))
        .route("/{module_type}/call", post(call_module_function))
        .route("/{module_type}/stream", post(stream_module_function))
        .fallback(not_found)
}

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
    Path(module_type): Path<ModuleType>,
) -> Result<Json<ModuleFunctions>> {
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
    Path(module_type): Path<ModuleType>,
    Json(request): Json<CallRequest>,
) -> Result<Json<CallResponse>> {
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

/// 流式调用指定模块的方法 (SSE)
pub async fn stream_module_function(
    State(state): State<AppState>,
    Path(module_type): Path<ModuleType>,
    Json(request): Json<CallRequest>,
) -> Sse<impl futures::Stream<Item = std::result::Result<Event, Infallible>>> {
    let stream_result = state
        .module_manager
        .stream_call(&module_type, &request.func_name, request.param)
        .await;

    type StreamType = Pin<Box<dyn futures::Stream<Item = std::result::Result<Event, Infallible>> + Send>>;

    let stream: StreamType = match stream_result {
        Ok(mut chunk_stream) => {
            let stream = async_stream::stream! {
                while let Some(result) = chunk_stream.next().await {
                    match result {
                        Ok(chunk) => {
                            let data = serde_json::json!({
                                "content": chunk
                            });
                            yield Ok(Event::default()
                                .event("chunk")
                                .data(data.to_string()));
                        }
                        Err(e) => {
                            let data = serde_json::json!({
                                "error": e.to_string()
                            });
                            yield Ok(Event::default()
                                .event("error")
                                .data(data.to_string()));
                            break;
                        }
                    }
                }
                // Send done event
                let data = serde_json::json!({"content": ""});
                yield Ok(Event::default()
                    .event("done")
                    .data(data.to_string()));
            };
            Box::pin(stream) as StreamType
        }
        Err(e) => {
            let stream = async_stream::stream! {
                let data = serde_json::json!({
                    "error": e.to_string()
                });
                yield Ok(Event::default()
                    .event("error")
                    .data(data.to_string()));
            };
            Box::pin(stream) as StreamType
        }
    };

    Sse::new(stream)
}
