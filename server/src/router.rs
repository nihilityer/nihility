mod embed_assets;
mod html_page;
mod html_page_manager;
mod jwt;
mod module_config;
mod module_manager;
mod test;

use crate::error::*;
use crate::router::embed_assets::embed_assets_handler;
use crate::router::html_page::get_html_page;
use crate::router::html_page_manager::html_page_manager_router;
use crate::router::jwt::{auth_middleware, authorize};
use crate::router::module_config::module_config_router;
use crate::router::module_manager::module_manager_router;
use crate::router::test::test;
use crate::AppState;
use axum::routing::{any, get, post};
use axum::{middleware, Router};
pub(crate) use jwt::JwtKeys;

pub(super) fn app_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/auth", post(authorize))
        .nest(
            "/api",
            Router::new()
                .route("/test", get(test))
                .nest("/modules", module_manager_router())
                .nest("/module-configs", module_config_router())
                .nest("/html-pages", html_page_manager_router())
                .fallback(any(not_found))
                .layer(middleware::from_fn_with_state(
                    state.jwt.clone(),
                    auth_middleware,
                )),
        )
        .route("/html/{path}", get(get_html_page))
        .fallback(get(embed_assets_handler))
}

async fn not_found() -> Result<()> {
    Err(NihilityServerError::NotFound("path".to_string()))
}
