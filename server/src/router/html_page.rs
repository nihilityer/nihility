use crate::error::*;
use crate::service::HtmlPageService;
use crate::AppState;
use axum::extract::{Path, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::Response;

pub(super) async fn get_html_page(
    state: State<AppState>,
    Path(path): Path<String>,
) -> Result<Response<String>> {
    Ok(Response::builder()
        .header(CONTENT_TYPE, "text/html")
        .body(HtmlPageService::find_html_by_path(&state.conn, &path).await?)?)
}
