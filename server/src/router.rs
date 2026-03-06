use axum::body::Body;
use axum::Router;
use axum::http::header::CONTENT_TYPE;
use axum::http::{Response, StatusCode};
use axum::routing::get;
use mime_guess::from_path;
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "../frontend/dist"]
struct Assets;

pub(crate) fn app_router() -> Router {
    Router::new().fallback(get(get(static_handler)))
}

async fn static_handler(uri: axum::http::Uri) -> Result<Response<Body>, StatusCode> {
    let path = uri.path().trim_start_matches('/');
    let path = if path.is_empty() || Assets::get(path).is_none() {
        "index.html"
    } else {
        path
    };

    match Assets::get(path) {
        Some(content) => {
            let mime = from_path(path).first_or_octet_stream();
            Ok(Response::builder()
                .header(CONTENT_TYPE, mime.as_ref())
                .body(Body::from(content.data.into_owned()))
                .unwrap())
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}
