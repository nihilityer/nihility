mod chromium_install;
mod deps_install;
pub mod error;
mod model_install;

use crate::chromium_install::chromium_install;
use crate::deps_install::deps_install;
use crate::model_install::model_install;
use error::*;
use tokio::try_join;
use tracing::{error, info};

pub async fn init() -> Result<()> {
    match deps_install().await {
        Ok(()) => {
            info!("System dependencies check complete");
        }
        Err(err) => {
            error!("Failed to install system dependencies: {}", err);
            return Err(err);
        }
    }
    let model_install_task = tokio::spawn(model_install());
    let chromium_install_task = tokio::spawn(chromium_install());
    match try_join!(model_install_task, chromium_install_task) {
        Ok((Ok(()), Ok(()))) => {
            info!("Initialization complete");
        }
        Ok((Err(err), Ok(_))) => {
            error!("Failed to initialize model: {}", err);
            return Err(err);
        }
        Ok((Ok(()), Err(err))) => {
            error!("Failed to initialize chromium: {}", err);
            return Err(err);
        }
        Ok((Err(model_err), Err(chromium_err))) => {
            error!("Failed to initialize model: {}", model_err);
            error!("Failed to initialize chromium: {}", chromium_err);
            return Err(InitError::Unknown(format!(
                "Failed to initialize model and chromium: model_err: {}, chromium_err: {}",
                model_err, chromium_err
            )));
        }
        Err(e) => {
            error!("Failed to exec init task: {}", e);
            return Err(InitError::Task(e));
        }
    }
    Ok(())
}
