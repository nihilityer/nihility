use crate::error::InitError;
use std::process::Command;
use tracing::info;

fn run_cmd(cmd: &str, args: &[&str]) -> Result<(), InitError> {
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| InitError::Chromium(e.to_string()))?;
    if !output.status.success() {
        return Err(InitError::Chromium(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    Ok(())
}

fn run_sh(cmd: &str) -> Result<(), InitError> {
    let output = Command::new("sh")
        .args(["-c", cmd])
        .output()
        .map_err(|e| InitError::Chromium(e.to_string()))?;
    if !output.status.success() {
        return Err(InitError::Chromium(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ));
    }
    Ok(())
}

pub async fn chromium_install() -> Result<(), InitError> {
    info!("Installing wget and gnupg...");
    run_cmd("apt-get", &["update"])?;
    run_cmd(
        "apt-get",
        &["install", "-y", "--no-install-recommends", "wget", "gnupg"],
    )?;
    info!("wget and gnupg installed");

    info!("Adding Google Chrome GPG key...");
    run_sh(
        "wget -qO- https://dl.google.com/linux/linux_signing_key.pub | gpg --dearmor -o /usr/share/keyrings/google-chrome.gpg",
    )?;
    info!("GPG key added");

    info!("Adding Google Chrome repository...");
    run_sh(
        "echo 'deb [arch=amd64 signed-by=/usr/share/keyrings/google-chrome.gpg] http://dl.google.com/linux/chrome/deb/ stable main' > /etc/apt/sources.list.d/google-chrome.list",
    )?;
    run_cmd("apt-get", &["update"])?;
    info!("Repository added");

    info!("Installing google-chrome-stable...");
    run_cmd(
        "apt-get",
        &[
            "install",
            "-y",
            "--no-install-recommends",
            "google-chrome-stable",
        ],
    )?;
    info!("google-chrome-stable installed successfully");

    Ok(())
}
