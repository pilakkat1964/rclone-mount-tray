// Data models for rclone configuration and mounts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported cloud services
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CloudService {
    GoogleDrive,
    OneDrive,
    Dropbox,
    AmazonS3,
    BackBlaze,
    Box,
}

impl CloudService {
    pub fn as_str(&self) -> &str {
        match self {
            CloudService::GoogleDrive => "drive",
            CloudService::OneDrive => "onedrive",
            CloudService::Dropbox => "dropbox",
            CloudService::AmazonS3 => "s3",
            CloudService::BackBlaze => "b2",
            CloudService::Box => "box",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            CloudService::GoogleDrive => "Google Drive",
            CloudService::OneDrive => "Microsoft OneDrive",
            CloudService::Dropbox => "Dropbox",
            CloudService::AmazonS3 => "Amazon S3",
            CloudService::BackBlaze => "Backblaze B2",
            CloudService::Box => "Box.com",
        }
    }

    pub fn icon_name(&self) -> &str {
        match self {
            CloudService::GoogleDrive => "folder-remote",
            CloudService::OneDrive => "folder-remote",
            CloudService::Dropbox => "folder-remote",
            CloudService::AmazonS3 => "folder-remote",
            CloudService::BackBlaze => "folder-remote",
            CloudService::Box => "folder-remote",
        }
    }

    pub fn icon_char(&self) -> &str {
        match self {
            CloudService::GoogleDrive => "🔵",
            CloudService::OneDrive => "🔷",
            CloudService::Dropbox => "🔹",
            CloudService::AmazonS3 => "📦",
            CloudService::BackBlaze => "💾",
            CloudService::Box => "📁",
        }
    }
}

/// Rclone remote configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    pub name: String,
    pub service: CloudService,
    pub remote_path: Option<String>,
    pub auth_method: String,
    pub credentials: AuthCredentials,
    pub properties: HashMap<String, String>,
}

impl RemoteConfig {
    pub fn new(name: String, service: CloudService) -> Self {
        Self {
            name,
            service,
            remote_path: None,
            auth_method: "oauth".to_string(),
            credentials: AuthCredentials::new(service),
            properties: HashMap::new(),
        }
    }

    pub fn set_property(&mut self, key: String, value: String) {
        self.properties.insert(key, value);
    }

    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }
}

/// Mount configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountConfig {
    pub id: String,
    pub name: String,
    pub remote_name: String,
    pub mount_point: String,
    pub options: MountOptions,
    pub auto_mount: bool,
    pub enabled: bool,
}

impl MountConfig {
    pub fn new(name: String, remote_name: String, mount_point: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            remote_name,
            mount_point,
            options: MountOptions::default(),
            auto_mount: false,
            enabled: true,
        }
    }
}

/// Mount options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountOptions {
    pub allow_non_empty: bool,
    pub allow_other: bool,
    pub read_only: bool,
    pub cache_dir: Option<String>,
    pub poll_interval: Option<String>,
}

impl Default for MountOptions {
    fn default() -> Self {
        Self {
            allow_non_empty: false,
            allow_other: false,
            read_only: false,
            cache_dir: None,
            poll_interval: None,
        }
    }
}

/// Authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCredentials {
    pub service: CloudService,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub token_expiry: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

impl AuthCredentials {
    pub fn new(service: CloudService) -> Self {
        Self {
            service,
            access_token: None,
            refresh_token: None,
            token_expiry: None,
            client_id: None,
            client_secret: None,
        }
    }
}

impl Default for AuthCredentials {
    fn default() -> Self {
        Self {
            service: CloudService::GoogleDrive,
            access_token: None,
            refresh_token: None,
            token_expiry: None,
            client_id: None,
            client_secret: None,
        }
    }
}

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub remotes: Vec<RemoteConfig>,
    pub mounts: Vec<MountConfig>,
    pub auth_cache: HashMap<String, String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            remotes: Vec::new(),
            mounts: Vec::new(),
            auth_cache: HashMap::new(),
        }
    }
}

/// Mount status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MountStatus {
    Mounted,
    Unmounted,
    Mounting,
    Unmounting,
    Error(String),
}

impl MountStatus {
    pub fn as_str(&self) -> &str {
        match self {
            MountStatus::Mounted => "Mounted",
            MountStatus::Unmounted => "Unmounted",
            MountStatus::Mounting => "Mounting...",
            MountStatus::Unmounting => "Unmounting...",
            MountStatus::Error(_) => "Error",
        }
    }
}
