use crate::AppState;
use crate::error::*;
use crate::service::HtmlPage;
use axum::extract::{Path, State};
use axum::http::Response;
use axum::http::header::CONTENT_TYPE;

pub(super) async fn get_html_page(
    state: State<AppState>,
    Path(path): Path<String>,
) -> Result<Response<String>> {
    Ok(Response::builder()
        .header(CONTENT_TYPE, "text/html")
        .body(HtmlPage::find_html_by_path(&state.conn, &path).await?)?)
}
