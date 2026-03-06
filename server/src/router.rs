mod embed_assets;
mod html_page;

use crate::AppState;
use crate::router::embed_assets::embed_assets_handler;
use crate::router::html_page::get_html_page;
use axum::Router;
use axum::routing::get;

pub(super) fn app_router() -> Router<AppState> {
    Router::new()
        .route("/html/{path}", get(get_html_page))
        .fallback(get(embed_assets_handler))
}
