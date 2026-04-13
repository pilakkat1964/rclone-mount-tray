// Mount management module
// Handles rclone mount configuration and status checking

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info};

/// Represents a single rclone mount configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct MountConfig {
    /// Remote name (e.g., "gdrive_pilakkat")
    pub remote: String,
    /// Mount point path (e.g., "/home/user/gdrive_pilakkat")
    pub mount_point: PathBuf,
}

/// Status of a mount
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountStatus {
    Mounted,
    Unmounted,
    Error,
}

impl std::fmt::Display for MountStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MountStatus::Mounted => write!(f, "Mounted"),
            MountStatus::Unmounted => write!(f, "Unmounted"),
            MountStatus::Error => write!(f, "Error"),
        }
    }
}

/// Information about a mount including its current status
#[derive(Debug, Clone)]
pub struct MountInfo {
    pub config: MountConfig,
    pub status: MountStatus,
    pub size: Option<String>,
}

/// Manager for rclone mounts
pub struct MountManager {
    mounts: Vec<MountConfig>,
    config_path: PathBuf,
}

impl MountManager {
    /// Create a new mount manager, loading configuration from the default location
    pub fn new() -> Result<Self> {
        let config_path = Self::default_config_path();
        debug!("Loading mount configuration from: {:?}", config_path);

        let mounts = if config_path.exists() {
            Self::load_config_file(&config_path)?
        } else {
            info!("No config file found, using hardcoded defaults");
            // Hardcoded defaults matching the bash script
            vec![
                MountConfig {
                    remote: "gdrive_pilakkat".to_string(),
                    mount_point: dirs::home_dir()
                        .context("Could not determine home directory")?
                        .join("gdrive_pilakkat"),
                },
                MountConfig {
                    remote: "gdrive_goofybits".to_string(),
                    mount_point: dirs::home_dir()
                        .context("Could not determine home directory")?
                        .join("gdrive_goofybits"),
                },
            ]
        };

        Ok(Self {
            mounts,
            config_path,
        })
    }

    /// Get the default configuration path
    fn default_config_path() -> PathBuf {
        dirs::config_dir()
            .map(|p| p.join("rclone-mount-tray/mounts.toml"))
            .unwrap_or_else(|| PathBuf::from(".rclone-mount-tray.toml"))
    }

    /// Load configuration from a TOML file
    fn load_config_file(path: &Path) -> Result<Vec<MountConfig>> {
        let contents =
            fs::read_to_string(path).context("Failed to read mount configuration file")?;
        let config: TomlMountConfig =
            toml::from_str(&contents).context("Failed to parse mount configuration")?;
        Ok(config.mounts)
    }

    /// Save configuration to file
    pub fn save_config(&self) -> Result<()> {
        let config = TomlMountConfig {
            mounts: self.mounts.clone(),
        };
        let config_path = &self.config_path;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let contents = toml::to_string_pretty(&config).context("Failed to serialize config")?;
        fs::write(config_path, contents).context("Failed to write configuration file")?;

        Ok(())
    }

    /// Get all configured mounts
    pub fn get_mounts(&self) -> Vec<MountConfig> {
        self.mounts.clone()
    }

    /// Check if a mount point is currently mounted
    pub fn check_mount_status(&self, mount_point: &Path) -> MountStatus {
        match check_mountpoint(mount_point) {
            Ok(is_mounted) => {
                if is_mounted {
                    MountStatus::Mounted
                } else {
                    MountStatus::Unmounted
                }
            }
            Err(e) => {
                error!("Error checking mount status for {:?}: {}", mount_point, e);
                MountStatus::Error
            }
        }
    }

    /// Get information about all mounts including their status
    pub fn get_mount_info(&self) -> Vec<MountInfo> {
        self.mounts
            .iter()
            .map(|config| {
                let status = self.check_mount_status(&config.mount_point);
                let size = if status == MountStatus::Mounted {
                    get_mount_size(&config.mount_point).ok()
                } else {
                    None
                };

                MountInfo {
                    config: config.clone(),
                    status,
                    size,
                }
            })
            .collect()
    }

    /// Add a new mount configuration
    pub fn add_mount(&mut self, remote: String, mount_point: PathBuf) -> Result<()> {
        let config = MountConfig {
            remote,
            mount_point,
        };

        if !self.mounts.contains(&config) {
            self.mounts.push(config);
            self.save_config()?;
        }

        Ok(())
    }

    /// Remove a mount configuration
    pub fn remove_mount(&mut self, remote: &str) -> Result<()> {
        self.mounts.retain(|m| m.remote != remote);
        self.save_config()?;
        Ok(())
    }
}

impl Default for MountManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default MountManager")
    }
}

/// Helper struct for TOML serialization
#[derive(Debug, Serialize, Deserialize)]
struct TomlMountConfig {
    mounts: Vec<MountConfig>,
}

/// Check if a mount point is currently mounted using /proc/mounts
fn check_mountpoint(mount_point: &Path) -> Result<bool> {
    let mount_point_str = mount_point
        .to_str()
        .context("Mount point path contains invalid UTF-8")?;

    let mounts_content =
        fs::read_to_string("/proc/mounts").context("Failed to read /proc/mounts")?;

    for line in mounts_content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == mount_point_str {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Get the size of a mounted filesystem
fn get_mount_size(mount_point: &Path) -> Result<String> {
    use std::process::Command;

    let output = Command::new("du")
        .args(&["-sh", mount_point.to_str().unwrap_or(".")])
        .output()
        .context("Failed to run du command")?;

    if output.status.success() {
        let size_str = String::from_utf8(output.stdout).context("Failed to parse du output")?;
        Ok(size_str
            .split_whitespace()
            .next()
            .unwrap_or("?")
            .to_string())
    } else {
        Ok("?".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mount_status_display() {
        assert_eq!(MountStatus::Mounted.to_string(), "Mounted");
        assert_eq!(MountStatus::Unmounted.to_string(), "Unmounted");
        assert_eq!(MountStatus::Error.to_string(), "Error");
    }

    #[test]
    fn test_mount_config_equality() {
        let config1 = MountConfig {
            remote: "gdrive".to_string(),
            mount_point: PathBuf::from("/home/user/gdrive"),
        };

        let config2 = MountConfig {
            remote: "gdrive".to_string(),
            mount_point: PathBuf::from("/home/user/gdrive"),
        };

        assert_eq!(config1, config2);
    }
}
