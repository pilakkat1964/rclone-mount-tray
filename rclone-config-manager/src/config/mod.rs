// Rclone configuration file management

use crate::models::{CloudService, RemoteConfig};
use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct RcloneConfigManager {
    config_path: PathBuf,
}

impl RcloneConfigManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?
            .join("rclone");

        fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("rclone.conf");

        Ok(Self { config_path })
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Read existing rclone configuration file
    pub fn read_config(&self) -> Result<String> {
        if self.config_path.exists() {
            fs::read_to_string(&self.config_path)
                .map_err(|e| anyhow!("Failed to read rclone config: {}", e))
        } else {
            Ok(String::new())
        }
    }

    /// Parse remote configuration from rclone.conf
    pub fn parse_remotes(&self) -> Result<Vec<RemoteConfig>> {
        let content = self.read_config()?;
        let mut remotes = Vec::new();

        let mut current_section: Option<String> = None;
        let mut current_config: Option<RemoteConfig> = None;

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
                continue;
            }

            // Section header [remote_name]
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                // Save previous remote if exists
                if let Some(config) = current_config.take() {
                    remotes.push(config);
                }

                current_section = Some(trimmed[1..trimmed.len() - 1].to_string());
                continue;
            }

            // Key = value
            if let Some(section) = &current_section {
                if let Some((key, value)) = trimmed.split_once('=') {
                    let key = key.trim().to_string();
                    let value = value.trim().to_string();

                    if current_config.is_none() {
                        // Detect service type from config
                        let service = self.detect_service_type(&key, &value);
                        let mut config = RemoteConfig::new(section.clone(), service);
                        config.set_property(key, value);
                        current_config = Some(config);
                    } else if let Some(config) = &mut current_config {
                        config.set_property(key, value);
                    }
                }
            }
        }

        // Add last remote
        if let Some(config) = current_config {
            remotes.push(config);
        }

        Ok(remotes)
    }

    /// Detect cloud service from rclone config properties
    fn detect_service_type(&self, key: &str, _value: &str) -> CloudService {
        let key_lower = key.to_lowercase();

        if key_lower.contains("type") {
            // Will be set properly based on value
            CloudService::GoogleDrive
        } else if key_lower.contains("drive") {
            CloudService::GoogleDrive
        } else if key_lower.contains("onedrive") || key_lower.contains("one_drive") {
            CloudService::OneDrive
        } else {
            CloudService::GoogleDrive
        }
    }

    /// Add a new remote to rclone.conf
    pub fn add_remote(&self, remote: RemoteConfig) -> Result<()> {
        // Check if remote already exists
        let existing = self.parse_remotes()?;
        if existing.iter().any(|r| r.name == remote.name) {
            return Err(anyhow!("Remote '{}' already exists", remote.name));
        }

        self.write_remote(&remote)
    }

    /// Update an existing remote in rclone.conf
    pub fn update_remote(&self, remote: RemoteConfig) -> Result<()> {
        self.write_remote(&remote)
    }

    /// Write remote configuration to rclone.conf
    pub fn write_remote(&self, remote: &RemoteConfig) -> Result<()> {
        let mut content = self.read_config()?;

        // Remove existing section if present
        let mut lines: Vec<String> = content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.starts_with(&format!("[{}]", remote.name))
                    && !(trimmed.starts_with('[') && trimmed.ends_with(']'))
            })
            .map(|s| s.to_string())
            .collect();

        // Add new section
        if !lines.is_empty() && !lines.last().unwrap().is_empty() {
            lines.push(String::new());
        }

        lines.push(format!("[{}]", remote.name));
        lines.push(format!("type = {}", remote.service.as_str()));

        for (key, value) in &remote.properties {
            if key != "type" {
                lines.push(format!("{} = {}", key, value));
            }
        }

        let new_content = lines.join("\n");
        fs::write(&self.config_path, new_content)
            .map_err(|e| anyhow!("Failed to write rclone config: {}", e))
    }

    /// Remove a remote from rclone.conf
    pub fn remove_remote(&self, remote_name: &str) -> Result<()> {
        let content = self.read_config()?;
        let mut in_section = false;
        let mut section_name = String::new();

        let filtered: String = content
            .lines()
            .filter(|line| {
                let trimmed = line.trim();

                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    section_name = trimmed[1..trimmed.len() - 1].to_string();
                    in_section = section_name == remote_name;

                    if in_section {
                        return false; // Skip section header
                    }
                }

                if in_section && trimmed.is_empty() {
                    in_section = false;
                }

                !in_section
            })
            .collect::<Vec<_>>()
            .join("\n");

        fs::write(&self.config_path, filtered)
            .map_err(|e| anyhow!("Failed to update rclone config: {}", e))
    }

    /// Create a backup of rclone.conf
    pub fn backup_config(&self) -> Result<PathBuf> {
        let backup_path = self.config_path.with_extension("conf.backup");
        fs::copy(&self.config_path, &backup_path)
            .map_err(|e| anyhow!("Failed to backup config: {}", e))?;
        Ok(backup_path)
    }
}
