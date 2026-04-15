mod chromium_install;
mod deps_install;
pub mod error;
mod model_install;

use crate::chromium_install::chromium_install;
use crate::deps_install::deps_install;
use crate::model_install::model_install;
use error::*;
use tokio::task::JoinSet;
use tracing::{error, info};

fn is_in_container() -> bool {
    std::env::var("NIHILITY_IN_CONTAINER").is_ok()
}

pub async fn init() -> Result<()> {
    if is_in_container() {
        match deps_install().await {
            Ok(()) => {
                info!("System dependencies check complete");
            }
            Err(err) => {
                error!("Failed to install system dependencies: {}", err);
                return Err(err);
            }
        }
    } else {
        info!("Not in container, skipping system dependencies installation");
    }

    let mut task_set = JoinSet::new();
    task_set.spawn(model_install());

    if is_in_container() {
        task_set.spawn(chromium_install());
    }

    while let Some(result) = task_set.join_next().await {
        match result {
            Ok(Ok(())) => {
                info!("Task completed successfully");
            }
            Ok(Err(err)) => {
                error!("Task failed: {}", err);
                return Err(err);
            }
            Err(e) => {
                error!("Task panicked: {}", e);
                return Err(InitError::Task(e));
            }
        }
    }

    info!("Initialization complete");
    Ok(())
}
