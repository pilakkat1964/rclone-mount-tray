mod mount_manager;
mod systemd_manager;
mod tray_ui;

use anyhow::Result;
use mount_manager::MountManager;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tray_ui::TrayUI;
use tracing::{error, info};

/// Main application state
pub struct App {
    mount_manager: Mutex<MountManager>,
    tray_ui: Mutex<TrayUI>,
    last_refresh: Mutex<Instant>,
}

impl App {
    /// Create a new application instance
    fn new() -> Result<Self> {
        // Initialize tracing/logging
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_target(false)
            .with_thread_ids(false)
            .init();

        info!("Initializing RClone Mount Manager Tray Application");

        // Create mount manager
        let mount_manager = MountManager::new()?;

        // Create tray UI (without event proxy - we'll use blocking for now)
        let tray_ui = TrayUI::new()?;

        info!("Application initialized successfully");

        Ok(Self {
            mount_manager: Mutex::new(mount_manager),
            tray_ui: Mutex::new(tray_ui),
            last_refresh: Mutex::new(Instant::now()),
        })
    }

    /// Refresh mount status and update UI
    fn refresh_status(&self) -> Result<()> {
        let mut manager = self.mount_manager.lock().unwrap();
        let mounts = manager.get_mount_info();

        let mut ui = self.tray_ui.lock().unwrap();
        ui.update_menu(&mounts)?;
        ui.update_icon(&mounts)?;

        let mut last_refresh = self.last_refresh.lock().unwrap();
        *last_refresh = Instant::now();

        Ok(())
    }

    /// Handle a mount toggle action
    fn toggle_mount(&self, remote: &str) -> Result<()> {
        info!("Toggling mount for remote: {}", remote);

        let manager = self.mount_manager.lock().unwrap();
        let mounts = manager.get_mount_info();

        // Find the mount
        let mount = mounts
            .iter()
            .find(|m| m.config.remote == remote)
            .ok_or_else(|| anyhow::anyhow!("Mount not found: {}", remote))?;

        // Toggle based on current status
        match mount.status {
            mount_manager::MountStatus::Mounted => {
                info!("Unmounting: {}", remote);
                // Run in blocking context
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(systemd_manager::SystemdManager::stop_service(remote))?;
                info!("Successfully unmounted: {}", remote);
            }
            mount_manager::MountStatus::Unmounted => {
                info!("Mounting: {}", remote);
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(systemd_manager::SystemdManager::start_service(remote))?;
                info!("Successfully mounted: {}", remote);
            }
            mount_manager::MountStatus::Error => {
                info!("Attempting to recover from error state by mounting: {}", remote);
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(systemd_manager::SystemdManager::start_service(remote))?;
            }
        }

        drop(manager);

        // Update UI after toggle
        self.refresh_status()?;

        Ok(())
    }
}

fn main() -> Result<()> {
    let app = App::new()?;

    // Initial refresh
    app.refresh_status()?;

    // Periodic refresh loop using standard library sleep
    loop {
        std::thread::sleep(Duration::from_secs(5));

        if let Err(e) = app.refresh_status() {
            error!("Failed to refresh mount status: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = App::new();
        assert!(app.is_ok());
    }
}
