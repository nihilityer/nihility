use crate::error::*;
use crate::router::not_found;
use crate::service::ModuleConfigService;
use crate::AppState;
use axum::extract::{Path, State};
use axum::routing::{get, put};
use axum::{Json, Router};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub fn module_config_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_configs))
        .route("/{module_name}", get(get_config_by_module_name))
        .route("/id/{id}", put(update_config))
        .fallback(not_found)
}

/// 模块配置摘要（列表用）
#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleConfigSummary {
    pub id: Uuid,
    pub module_name: String,
    pub updated_at: DateTime<FixedOffset>,
}

/// 模块配置列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleConfigListResponse {
    pub configs: Vec<ModuleConfigSummary>,
    pub total: u64,
}

/// 模块配置完整响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ModuleConfigResponse {
    pub id: Uuid,
    pub module_name: String,
    pub config_value: Value,
    pub json_schema: Value,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

/// 模块配置更新请求
#[derive(Debug, Deserialize)]
pub struct ModuleConfigUpdateRequest {
    pub config_value: Value,
}

/// 获取所有模块配置列表
pub async fn list_configs(
    State(state): State<AppState>,
) -> Result<Json<ModuleConfigListResponse>> {
    let configs = ModuleConfigService::list_all(&state.conn).await?;
    let total = configs.len() as u64;

    let summaries: Vec<ModuleConfigSummary> = configs
        .into_iter()
        .map(|config| ModuleConfigSummary {
            id: config.id,
            module_name: config.module_name,
            updated_at: config.updated_at,
        })
        .collect();

    Ok(Json(ModuleConfigListResponse { configs: summaries, total }))
}

/// 根据模块名称获取配置
pub async fn get_config_by_module_name(
    State(state): State<AppState>,
    Path(module_name): Path<String>,
) -> Result<Json<ModuleConfigResponse>> {
    let config = ModuleConfigService::find_by_module_name(&state.conn, &module_name).await?;

    Ok(Json(ModuleConfigResponse {
        id: config.id,
        module_name: config.module_name,
        config_value: config.config_value,
        json_schema: config.json_schema,
        created_at: config.created_at,
        updated_at: config.updated_at,
    }))
}

/// 根据 ID 更新模块配置
pub async fn update_config(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<ModuleConfigUpdateRequest>,
) -> Result<Json<ModuleConfigResponse>> {
    let config = ModuleConfigService::update(&state.conn, &id, request.config_value).await?;

    Ok(Json(ModuleConfigResponse {
        id: config.id,
        module_name: config.module_name,
        config_value: config.config_value,
        json_schema: config.json_schema,
        created_at: config.created_at,
        updated_at: config.updated_at,
    }))
}
