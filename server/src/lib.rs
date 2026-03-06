pub mod error;
mod router;
mod service;

use crate::error::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use nihility_server_migration::{Migrator, MigratorTrait};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    addr: String,
    port: u16,
    database: DatabaseConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    database_url: String,
    max_connections: u32,
    min_connections: u32,
    connect_timeout: u64,
    acquire_timeout: u64,
    idle_timeout: u64,
    max_lifetime: u64,
}

#[derive(Clone)]
struct AppState {
    conn: DatabaseConnection,
}

pub async fn start_server(config: ServerConfig) -> Result<()> {
    let mut opt = ConnectOptions::new(config.database.database_url);
    opt.max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .connect_timeout(Duration::from_secs(config.database.connect_timeout))
        .acquire_timeout(Duration::from_secs(config.database.acquire_timeout))
        .idle_timeout(Duration::from_secs(config.database.idle_timeout))
        .max_lifetime(Duration::from_secs(config.database.max_lifetime))
        .sqlx_logging(false);
    let conn = Database::connect(opt).await?;
    Migrator::up(&conn, None).await?;

    let state = AppState { conn };
    let app = router::app_router().with_state(state);

    let listener = tokio::net::TcpListener::bind((config.addr, config.port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            addr: "127.0.0.1".to_string(),
            port: 8080,
            database: Default::default(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_url: "sqlite://db.sqlite?mode=rwc".to_string(),
            max_connections: 100,
            min_connections: 5,
            connect_timeout: 8,
            acquire_timeout: 8,
            idle_timeout: 8,
            max_lifetime: 8,
        }
    }
}
