mod router;
pub mod error;
use crate::error::*;

pub async fn start_server() -> Result<()> {
    let app = router::app_router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}