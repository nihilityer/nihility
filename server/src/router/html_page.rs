use crate::error::*;
use crate::AppState;
use axum::extract::{Path, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::Response;
use nihility_store_operate::html_page;

pub(super) async fn get_html_page(
    state: State<AppState>,
    Path(path): Path<String>,
) -> Result<Response<String>> {
    Ok(Response::builder()
        .header(CONTENT_TYPE, "text/html")
        .body(html_page::find_html_by_path(&state.conn, &path).await?)?)
}
