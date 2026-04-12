use crate::error::*;
use crate::router::not_found;
use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, FixedOffset};
use nihility_store_operate::html_page;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn html_page_manager_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_pages).post(create_page))
        .route("/{id}", get(get_page).put(update_page).delete(delete_page))
        .fallback(not_found)
}

/// HTML 页面列表摘要
#[derive(Debug, Serialize, Deserialize)]
pub struct HtmlPageSummary {
    pub id: Uuid,
    pub path: String,
    pub update_at: DateTime<FixedOffset>,
}

/// HTML 页面列表响应
#[derive(Debug, Serialize, Deserialize)]
pub struct HtmlPageListResponse {
    pub pages: Vec<HtmlPageSummary>,
    pub total: usize,
}

/// HTML 页面完整响应
#[derive(Debug, Serialize, Deserialize)]
pub struct HtmlPageResponse {
    pub id: Uuid,
    pub path: String,
    pub html: String,
    pub update_at: DateTime<FixedOffset>,
}

/// HTML 页面创建/更新请求
#[derive(Debug, Serialize, Deserialize)]
pub struct HtmlPageRequest {
    pub path: String,
    pub html: String,
}

impl HtmlPageRequest {
    /// 验证请求数据
    pub fn validate(&self) -> Result<()> {
        // 路径不能包含 '..' 防止路径穿越
        if self.path.contains("..") {
            return Err(NihilityServerError::Config(
                "Path cannot contain '..'".to_string(),
            ));
        }

        // 路径长度限制
        if self.path.len() > 255 {
            return Err(NihilityServerError::Config(
                "Path is too long (max 255 characters)".to_string(),
            ));
        }

        // HTML 内容不能为空
        if self.html.trim().is_empty() {
            return Err(NihilityServerError::Config(
                "HTML content cannot be empty".to_string(),
            ));
        }

        // HTML 内容大小限制 (1MB)
        if self.html.len() > 1_048_576 {
            return Err(NihilityServerError::Config(
                "HTML content is too large (max 1MB)".to_string(),
            ));
        }

        Ok(())
    }
}

/// 获取所有 HTML 页面列表
pub async fn list_pages(State(state): State<AppState>) -> Result<Json<HtmlPageListResponse>> {
    let pages = html_page::list_all(&state.conn).await?;
    let total = pages.len();

    let summaries = pages
        .into_iter()
        .map(|page| HtmlPageSummary {
            id: page.id,
            path: page.path,
            update_at: page.update_at,
        })
        .collect();

    Ok(Json(HtmlPageListResponse {
        pages: summaries,
        total,
    }))
}

/// 根据 ID 获取单个 HTML 页面
pub async fn get_page(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<HtmlPageResponse>> {
    let page = html_page::find_by_id(&state.conn, &id).await?;

    Ok(Json(HtmlPageResponse {
        id: page.id,
        path: page.path,
        html: page.html,
        update_at: page.update_at,
    }))
}

/// 创建新的 HTML 页面
pub async fn create_page(
    State(state): State<AppState>,
    Json(request): Json<HtmlPageRequest>,
) -> Result<Json<HtmlPageResponse>> {
    request.validate()?;

    let page = html_page::create(&state.conn, request.path, request.html).await?;

    Ok(Json(HtmlPageResponse {
        id: page.id,
        path: page.path,
        html: page.html,
        update_at: page.update_at,
    }))
}

/// 更新现有 HTML 页面
pub async fn update_page(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<HtmlPageRequest>,
) -> Result<Json<HtmlPageResponse>> {
    request.validate()?;

    let page = html_page::update(&state.conn, &id, request.path, request.html).await?;

    Ok(Json(HtmlPageResponse {
        id: page.id,
        path: page.path,
        html: page.html,
        update_at: page.update_at,
    }))
}

/// 删除 HTML 页面
pub async fn delete_page(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode> {
    html_page::delete(&state.conn, &id).await?;
    Ok(StatusCode::NO_CONTENT)
}
