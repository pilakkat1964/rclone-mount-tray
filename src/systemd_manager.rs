// Systemd integration module
// Handles communication with systemd user service for mount/unmount operations

use anyhow::{anyhow, Context, Result};
use std::process::Command;
use tracing::{debug, error, info};

/// Manager for systemd user service operations
pub struct SystemdManager;

impl SystemdManager {
    /// Start a systemd user service (mount)
    pub async fn start_service(remote: &str) -> Result<()> {
        let service_name = format!("rclone-gdrive-{}.service", remote);
        info!("Starting systemd service: {}", service_name);

        let output = Command::new("systemctl")
            .args(&["--user", "start", &service_name])
            .output()
            .context("Failed to execute systemctl start")?;

        if output.status.success() {
            debug!("Successfully started service: {}", service_name);
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to start service {}: {}", service_name, stderr);
            Err(anyhow!("Failed to start service: {}", stderr))
        }
    }

    /// Stop a systemd user service (unmount)
    pub async fn stop_service(remote: &str) -> Result<()> {
        let service_name = format!("rclone-gdrive-{}.service", remote);
        info!("Stopping systemd service: {}", service_name);

        let output = Command::new("systemctl")
            .args(&["--user", "stop", &service_name])
            .output()
            .context("Failed to execute systemctl stop")?;

        if output.status.success() {
            debug!("Successfully stopped service: {}", service_name);
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("Failed to stop service {}: {}", service_name, stderr);
            Err(anyhow!("Failed to stop service: {}", stderr))
        }
    }

    /// Get the status of a systemd user service
    pub async fn get_service_status(remote: &str) -> Result<ServiceStatus> {
        let service_name = format!("rclone-gdrive-{}.service", remote);
        debug!("Checking status of service: {}", service_name);

        let output = Command::new("systemctl")
            .args(&["--user", "is-active", &service_name])
            .output()
            .context("Failed to execute systemctl is-active")?;

        let status_str = String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string();

        let status = match status_str.as_str() {
            "active" => ServiceStatus::Active,
            "inactive" => ServiceStatus::Inactive,
            "activating" => ServiceStatus::Activating,
            "deactivating" => ServiceStatus::Deactivating,
            _ => ServiceStatus::Failed,
        };

        debug!("Service {} status: {:?}", service_name, status);
        Ok(status)
    }
}

/// Systemd service states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceStatus {
    Active,
    Inactive,
    Activating,
    Deactivating,
    Failed,
}

impl std::fmt::Display for ServiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceStatus::Active => write!(f, "Active"),
            ServiceStatus::Inactive => write!(f, "Inactive"),
            ServiceStatus::Activating => write!(f, "Activating"),
            ServiceStatus::Deactivating => write!(f, "Deactivating"),
            ServiceStatus::Failed => write!(f, "Failed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_status_display() {
        assert_eq!(ServiceStatus::Active.to_string(), "Active");
        assert_eq!(ServiceStatus::Inactive.to_string(), "Inactive");
    }
}
