// Systemd service management for rclone mounts

use anyhow::{anyhow, Result};
use std::process::Command;

/// Manage rclone mount systemd services
pub struct SystemdManager;

impl SystemdManager {
    /// Generate systemd service name for a mount
    pub fn service_name(remote: &str, mount_point: &str) -> String {
        let sanitized = mount_point
            .replace('/', "-")
            .replace("~", "home")
            .trim_matches('-')
            .to_string();
        format!("rclone-mount-{}-{}.service", remote, sanitized)
    }

    /// Generate systemd service content
    pub fn generate_service(remote: &str, mount_point: &str, mount_options: &str) -> String {
        format!(
            r#"[Unit]
Description=RClone mount for {} at {}
After=network-online.target
Wants=network-online.target

[Service]
Type=notify
ExecStart=/usr/bin/rclone mount {} {} {}
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
"#,
            remote, mount_point, remote, mount_point, mount_options
        )
    }

    /// Start a mount service
    pub fn start_mount(service_name: &str) -> Result<()> {
        let output = Command::new("systemctl")
            .args(&["--user", "start", service_name])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to start service: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    /// Stop a mount service
    pub fn stop_mount(service_name: &str) -> Result<()> {
        let output = Command::new("systemctl")
            .args(&["--user", "stop", service_name])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to stop service: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    /// Check if mount is running
    pub fn is_mounted(service_name: &str) -> Result<bool> {
        let output = Command::new("systemctl")
            .args(&["--user", "is-active", service_name])
            .output()?;

        Ok(output.status.success())
    }

    /// Get service status
    pub fn get_status(service_name: &str) -> Result<String> {
        let output = Command::new("systemctl")
            .args(&["--user", "status", service_name])
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Enable service to start on boot
    pub fn enable_service(service_name: &str) -> Result<()> {
        let output = Command::new("systemctl")
            .args(&["--user", "enable", service_name])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to enable service: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    /// Disable service from boot
    pub fn disable_service(service_name: &str) -> Result<()> {
        let output = Command::new("systemctl")
            .args(&["--user", "disable", service_name])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to disable service: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }

    /// List all rclone mount services
    pub fn list_services() -> Result<Vec<String>> {
        let output = Command::new("systemctl")
            .args(&["--user", "list-units", "--all", "--no-pager"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let services: Vec<String> = stdout
            .lines()
            .filter(|line| line.contains("rclone-mount"))
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                parts.first().map(|s| s.to_string())
            })
            .collect();

        Ok(services)
    }

    /// Reload systemd daemon
    pub fn reload_daemon() -> Result<()> {
        let output = Command::new("systemctl")
            .args(&["--user", "daemon-reload"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow!(
                "Failed to reload daemon: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        Ok(())
    }
}
