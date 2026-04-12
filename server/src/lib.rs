pub mod error;
mod router;

use crate::error::*;
use crate::router::JwtKeys;
use nihility_module_manager::ModuleManager;
use nihility_secret::generate_secret;
use nihility_store_migration::{Migrator, MigratorTrait};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    addr: String,
    port: u16,
    jwt_secret: String,
    jwt_expiration: usize,
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
pub struct AppState {
    jwt: JwtKeys,
    conn: DatabaseConnection,
    module_manager: Arc<ModuleManager>,
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

    let jwt = JwtKeys::new(config.jwt_secret.as_bytes(), config.jwt_expiration);

    let module_manager = Arc::new(ModuleManager::init_from_file_config(conn.clone()).await?);

    let state = AppState {
        conn,
        jwt,
        module_manager,
    };
    let app = router::app_router(state.clone()).with_state(state);

    let listener = tokio::net::TcpListener::bind((config.addr, config.port)).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            addr: "0.0.0.0".to_string(),
            port: 8080,
            jwt_secret: generate_secret(26),
            jwt_expiration: 60 * 24 * 7,
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
