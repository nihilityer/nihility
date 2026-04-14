use crate::error::*;
use tracing::{error, info};

const SYSTEM_DEPS: &[&str] = &[
    "ca-certificates",
    "curl",
    "libnss3",
    "libatk1.0-0",
    "libatk-bridge2.0-0",
    "libcups2",
    "libdrm2",
    "libxkbcommon0",
    "libxcomposite1",
    "libxdamage1",
    "libxfixes3",
    "libxrandr2",
    "libgbm1",
    "libasound2",
    "libpango-1.0-0",
    "libpangocairo-1.0-0",
    "fonts-liberation",
    "xdg-utils",
];

fn is_root() -> bool {
    std::process::Command::new("id")
        .args(["-u"])
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim() == "0")
        .unwrap_or(false)
}

pub async fn deps_install() -> Result<()> {
    info!("Checking system dependencies");

    if !is_root() {
        info!("Not running as root, skipping system dependency installation");
        return Ok(());
    }

    info!("Updating package lists");
    let update_output = std::process::Command::new("apt-get")
        .args(["update"])
        .output()
        .map_err(|e| InitError::DepsInstall(format!("Failed to run apt-get update: {}", e)))?;

    if !update_output.status.success() {
        let stderr = String::from_utf8_lossy(&update_output.stderr);
        return Err(InitError::DepsInstall(format!(
            "apt-get update failed: {}",
            stderr
        )));
    }

    info!("Installing system dependencies");
    let install_output = std::process::Command::new("apt-get")
        .args(["install", "-y", "--no-install-recommends"])
        .args(SYSTEM_DEPS)
        .output()
        .map_err(|e| InitError::DepsInstall(format!("Failed to install dependencies: {}", e)))?;

    if !install_output.status.success() {
        let stderr = String::from_utf8_lossy(&install_output.stderr);
        return Err(InitError::DepsInstall(format!(
            "Failed to install system dependencies: {}",
            stderr
        )));
    }

    info!("System dependencies installed successfully");
    Ok(())
}
