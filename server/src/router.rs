mod embed_assets;
mod html_page;
mod jwt;
mod module_manager;
mod test;

use crate::error::*;
use crate::router::embed_assets::embed_assets_handler;
use crate::router::html_page::get_html_page;
use crate::router::jwt::{auth_middleware, authorize};
use crate::router::module_manager::{
    call_module_function, get_loaded_modules, query_all_functions, query_module_functions,
};
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
                .route("/modules", get(get_loaded_modules))
                .route("/modules/functions", get(query_all_functions))
                .route(
                    "/modules/{module_type}/functions",
                    get(query_module_functions),
                )
                .route("/modules/{module_type}/call", post(call_module_function))
                .fallback(any(not_found))
                .layer(middleware::from_fn_with_state(
                    state.jwt.clone(),
                    auth_middleware,
                )),
        )
        .route("/html/{path}", get(get_html_page))
        .fallback(get(embed_assets_handler))
}

async fn not_found(uri: http::Uri) -> Result<()> {
    Err(NihilityServerError::NotFound(uri.path().to_string()))
}
