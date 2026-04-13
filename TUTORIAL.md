# Rust GUI Application Development: A Comprehensive Tutorial

## Building a System Tray & GTK4 Application for Linux

This tutorial guides you through the complete design and implementation of a production-ready system tray application and modern GTK4 GUI using Rust. We'll cover architecture, component design, integration patterns, and best practices learned from building the RClone Mount Manager project.

---

## Table of Contents

1. [Introduction & Project Overview](#introduction--project-overview)
2. [Architecture & Design Patterns](#architecture--design-patterns)
3. [Setting Up Your Development Environment](#setting-up-your-development-environment)
4. [Part 1: Data Models & Domain Layer](#part-1-data-models--domain-layer)
5. [Part 2: Configuration Management](#part-2-configuration-management)
6. [Part 3: System Integration (Systemd)](#part-3-system-integration-systemd)
7. [Part 4: GTK4 UI Fundamentals](#part-4-gtk4-ui-fundamentals)
8. [Part 5: Building Dialogs & Widgets](#part-5-building-dialogs--widgets)
9. [Part 6: Event Handling & State Management](#part-6-event-handling--state-management)
10. [Part 7: System Tray Integration](#part-7-system-tray-integration)
11. [Part 8: Packaging & Distribution](#part-8-packaging--distribution)
12. [Appendix: Common Patterns & Troubleshooting](#appendix-common-patterns--troubleshooting)

---

## Introduction & Project Overview

### What You'll Learn

This tutorial teaches you to build:
- **A Modern GTK4 GUI Application** with multiple pages and dialogs
- **A System Tray Applet** that integrates with your desktop
- **Backend Services** for system integration (systemd, file I/O)
- **Professional Debian Packages** for distribution
- **Production-Ready Rust Code** with proper error handling

### The RClone Mount Manager Project

Our example project manages cloud storage mounts (Google Drive, OneDrive, etc.) through rclone. It consists of:

1. **Bash Mount Manager** - Command-line interface for on-demand mounting
2. **Rust Tray Applet** - System tray monitoring and quick control
3. **GTK4 GUI Manager** - Full-featured configuration interface (focus of this tutorial)
4. **Debian Package** - Professional distribution and installation

### Why Rust?

Rust is excellent for system programming because of:
- **Memory safety** without garbage collection
- **Fast execution** (native performance like C/C++)
- **Excellent error handling** with Result types
- **Great async/await** support for concurrent operations
- **Rich ecosystem** with high-quality libraries

---

## Architecture & Design Patterns

### High-Level Architecture

```
┌─────────────────────────────────────────────────────┐
│                   GTK4 Application                   │
│  ┌─────────────────────────────────────────────┐    │
│  │         UI Layer (gtk4, libadwaita)         │    │
│  │  ┌──────────────────────────────────────┐   │    │
│  │  │  Main Window | Dialogs | Widgets     │   │    │
│  │  └──────────────────────────────────────┘   │    │
│  └─────────────────────────────────────────────┘    │
│                       │                              │
│  ┌────────────────────┴─────────────────────────┐   │
│  │    Application Logic Layer                   │   │
│  │  ┌──────────────────────────────────────┐   │   │
│  │  │  Models | Event Handlers | State     │   │   │
│  │  └──────────────────────────────────────┘   │   │
│  └────────────────────┬─────────────────────────┘   │
│                       │                              │
│  ┌────────────────────┴─────────────────────────┐   │
│  │    Service Layer                            │   │
│  │  ┌──────────────────────────────────────┐   │   │
│  │  │  Config | Auth | Systemd | File I/O │   │   │
│  │  └──────────────────────────────────────┘   │   │
│  └────────────────────┬─────────────────────────┘   │
│                       │                              │
└───────────────────────┼──────────────────────────────┘
                        │
          ┌─────────────┼─────────────┐
          │             │             │
    ┌──────────┐  ┌──────────┐  ┌──────────┐
    │ rclone   │  │ systemd  │  │ OS Files │
    │ Remote   │  │ Services │  │ & Dirs   │
    │ Config   │  │          │  │          │
    └──────────┘  └──────────┘  └──────────┘
```

### Module Organization

```
src/
├── main.rs              # Application entry point
├── models/
│   └── mod.rs          # Data structures (CloudService, RemoteConfig, etc.)
├── config/
│   └── mod.rs          # RcloneConfigManager - read/write rclone.conf
├── auth/
│   └── mod.rs          # OAuth flows and token management
├── services/
│   └── mod.rs          # SystemdManager - systemd user service integration
└── ui/
    ├── mod.rs          # Main window and page composition
    ├── dialogs.rs      # Add/edit dialogs
    └── widgets.rs      # Reusable UI components
```

### Design Patterns Used

**1. Separation of Concerns**
- UI code stays in `ui/` module
- Business logic in `services/` and `config/`
- Data in `models/`

**2. Arc + Mutex for Shared State**
- Multiple UI components need access to config and services
- `Arc<Mutex<T>>` provides thread-safe shared ownership

**3. Error Handling with Result**
- All fallible operations return `Result<T, E>`
- Errors propagate up gracefully

**4. Builder Pattern**
- Application initialization builds all components
- Clear separation between construction and execution

---

## Setting Up Your Development Environment

### System Requirements

```bash
# Ubuntu/Debian
sudo apt-get update
sudo apt-get install -y \
    rustup \
    cargo \
    build-essential \
    pkg-config \
    libgtk-4-dev \
    libadwaita-1-dev \
    libssl-dev \
    systemd

# Verify Rust installation
rustc --version
cargo --version
```

### Project Setup

```bash
# Create new project
cargo new rclone-config-manager
cd rclone-config-manager

# Add dependencies
cargo add gtk4 --features v4_10
cargo add libadwaita --features v1_5
cargo add tokio --features rt-multi-thread,macros
cargo add serde --features derive
cargo add anyhow
cargo add tracing tracing-subscriber
cargo add uuid --features v4
cargo add dirs
```

### Verify Your Setup

Create a minimal GTK4 app to verify everything works:

```bash
# In Cargo.toml, set binary:
[[bin]]
name = "rclone-config-manager"
path = "src/main.rs"

# Create src/main.rs with minimal GTK app:
# (See Part 4 for the code)

cargo build
```

---

## Part 1: Data Models & Domain Layer

### Why Start with Models?

The domain layer defines what your application works with. By starting here, you:
- Clarify requirements and data structures
- Create a shared vocabulary for all modules
- Make the rest easier to implement

### The CloudService Enum

```rust
// src/models/mod.rs

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
    /// Get the rclone type identifier
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

    /// Get user-friendly display name
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

    /// Get emoji for UI display
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
```

**Key Concepts:**
- `#[derive(...)]` generates implementations automatically
- Pattern matching with `match` is exhaustive (compiler ensures all cases covered)
- Methods provide different representations for different purposes

### Remote Configuration Model

```rust
/// Rclone remote configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    pub name: String,                    // User-friendly name
    pub service: CloudService,           // Which cloud provider
    pub remote_path: Option<String>,     // Path within remote (e.g., /Archive)
    pub auth_method: String,             // "oauth" or "manual"
    pub credentials: AuthCredentials,    // Access tokens, etc.
    pub properties: HashMap<String, String>, // Service-specific config
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
```

**Pattern Used: Builder-like Constructor**
- `new()` creates struct with sensible defaults
- Methods allow flexible customization

### Mount Configuration Model

```rust
/// Mount configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountConfig {
    pub id: String,                // Unique identifier
    pub name: String,              // Display name
    pub remote_name: String,       // Which remote to mount
    pub mount_point: String,       // Where to mount (/home/user/gdrive)
    pub options: MountOptions,     // Mount-specific options
    pub auto_mount: bool,          // Mount on startup?
    pub enabled: bool,             // Is this mount active?
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

/// Mount-specific options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountOptions {
    pub allow_non_empty: bool,              // Allow mounting over non-empty dir
    pub allow_other: bool,                  // Allow other users to access
    pub read_only: bool,                    // Read-only mount
    pub cache_dir: Option<String>,          // Cache location
    pub poll_interval: Option<String>,      // Polling interval
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
```

**Key Concepts:**
- `Default` trait provides a sensible no-arg constructor
- `Option<T>` represents optional values (similar to nullable types in other languages)
- UUID for unique identifiers

### Mount Status Tracking

```rust
/// Mount status for real-time UI updates
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MountStatus {
    Mounted,                       // Mount is active
    Unmounted,                     // Mount is inactive
    Mounting,                      // Mount operation in progress
    Unmounting,                    // Unmount operation in progress
    Error(String),                 // Mount failed with error message
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

    /// Check if user can interact with this mount
    pub fn is_interactive(&self) -> bool {
        !matches!(self, MountStatus::Mounting | MountStatus::Unmounting)
    }
}
```

**Learning Point: Enums with Data**
- `Error(String)` holds the error message as associated data
- Pattern matching can extract the message: `Error(err) => println!("{}", err)`

### Authentication Credentials

```rust
/// Store authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthCredentials {
    pub service: CloudService,
    pub access_token: Option<String>,      // OAuth access token
    pub refresh_token: Option<String>,     // OAuth refresh token
    pub token_expiry: Option<String>,      // When token expires
    pub client_id: Option<String>,         // OAuth client ID
    pub client_secret: Option<String>,     // OAuth client secret
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

    /// Check if we have valid credentials
    pub fn is_authenticated(&self) -> bool {
        self.access_token.is_some()
    }

    /// Check if token is likely expired
    pub fn is_token_expired(&self) -> bool {
        // In real app, parse token_expiry and compare with current time
        false
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
```

### Why This Design?

✅ **Type Safety**: Compiler catches mistakes at compile time
✅ **Documentation**: Types are self-documenting
✅ **Reusability**: Models used across all layers
✅ **Serialization**: serde derives handle JSON/TOML/etc.
✅ **Pattern Matching**: Exhaustive checking with enums

---

## Part 2: Configuration Management

### Reading rclone.conf

The rclone configuration file is INI-format. Our manager needs to:
1. Parse existing config
2. Extract remotes
3. Write new remotes
4. Backup original

### Basic Config Manager Structure

```rust
// src/config/mod.rs

use crate::models::{CloudService, RemoteConfig};
use anyhow::{anyhow, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub struct RcloneConfigManager {
    config_path: PathBuf,
}

impl RcloneConfigManager {
    /// Initialize config manager
    pub fn new() -> Result<Self> {
        // Get config directory (typically ~/.config/rclone)
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?
            .join("rclone");

        // Create directory if it doesn't exist
        fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join("rclone.conf");

        Ok(Self { config_path })
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Read the entire rclone.conf file
    pub fn read_config(&self) -> Result<String> {
        if self.config_path.exists() {
            fs::read_to_string(&self.config_path)
                .map_err(|e| anyhow!("Failed to read rclone config: {}", e))
        } else {
            Ok(String::new())
        }
    }

    /// Write config back to file
    pub fn write_config(&self, content: &str) -> Result<()> {
        fs::write(&self.config_path, content)
            .map_err(|e| anyhow!("Failed to write rclone config: {}", e))
    }

    /// Create backup of current config
    pub fn backup_config(&self) -> Result<PathBuf> {
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_path = self.config_path
            .parent()
            .unwrap()
            .join(format!("rclone.conf.backup_{}", timestamp));

        fs::copy(&self.config_path, &backup_path)?;
        Ok(backup_path)
    }
}
```

**Key Pattern: Result Type for Error Handling**
- `Result<T>` = `Result<T, Error>`
- `?` operator short-circuits on error
- `anyhow!()` creates errors with formatted messages

### Parsing Remotes from Config

```rust
/// Parse remotes from rclone.conf
pub fn parse_remotes(&self) -> Result<Vec<RemoteConfig>> {
    let content = self.read_config()?;
    let mut remotes = Vec::new();

    let mut current_section: Option<String> = None;
    let mut current_config: Option<RemoteConfig> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with(';') {
            continue;
        }

        // Section headers: [remote-name]
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            // Save previous remote if any
            if let Some(config) = current_config.take() {
                remotes.push(config);
            }

            current_section = Some(
                trimmed[1..trimmed.len()-1].to_string()
            );

            // Create new remote config
            // Try to infer service type from properties
            current_config = Some(RemoteConfig::new(
                current_section.clone().unwrap(),
                CloudService::GoogleDrive, // Default, will update from properties
            ));

            continue;
        }

        // Property: key = value
        if let (Some(section), Some(ref mut config)) = (&current_section, &mut current_config) {
            if let Some((key, value)) = trimmed.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();

                // Determine service type from "type" property
                if key == "type" {
                    config.service = parse_service_type(&value);
                }

                config.set_property(key, value);
            }
        }
    }

    // Don't forget the last remote
    if let Some(config) = current_config {
        remotes.push(config);
    }

    Ok(remotes)
}

/// Parse service type from rclone type string
fn parse_service_type(type_str: &str) -> CloudService {
    match type_str {
        "drive" => CloudService::GoogleDrive,
        "onedrive" => CloudService::OneDrive,
        "dropbox" => CloudService::Dropbox,
        "s3" => CloudService::AmazonS3,
        "b2" => CloudService::BackBlaze,
        "box" => CloudService::Box,
        _ => CloudService::GoogleDrive,
    }
}
```

### Writing Remotes to Config

```rust
/// Write a remote configuration to rclone.conf
pub fn write_remote(&self, remote: &RemoteConfig) -> Result<()> {
    // Read current config
    let mut content = self.read_config()?;

    // Create backup before modifying
    self.backup_config()?;

    // Check if remote already exists
    let remote_header = format!("[{}]", remote.name);
    if let Some(start) = content.find(&remote_header) {
        // Find the next remote section or end of file
        let next_section = content[start + remote_header.len()..].find('[')
            .map(|pos| start + remote_header.len() + pos);

        // Remove old remote section
        if let Some(end) = next_section {
            content.drain(start..end);
        } else {
            content.truncate(start);
        }
    } else {
        // Add newline if file doesn't end with one
        if !content.ends_with('\n') {
            content.push('\n');
        }
    }

    // Append new remote section
    content.push_str(&format!("[{}]\n", remote.name));
    content.push_str(&format!("type = {}\n", remote.service.as_str()));

    for (key, value) in &remote.properties {
        content.push_str(&format!("{} = {}\n", key, value));
    }

    // Write back to file
    self.write_config(&content)?;
    Ok(())
}

/// Remove a remote from config
pub fn remove_remote(&self, remote_name: &str) -> Result<()> {
    let mut content = self.read_config()?;

    // Backup before modifying
    self.backup_config()?;

    let remote_header = format!("[{}]", remote_name);
    if let Some(start) = content.find(&remote_header) {
        let next_section = content[start..].find('\n')
            .and_then(|pos| content[start + pos..].find('['))
            .map(|pos| start + content[start..].find('\n').unwrap() + pos);

        if let Some(end) = next_section {
            content.drain(start..end);
        } else {
            content.truncate(start);
        }

        self.write_config(&content)?;
    }

    Ok(())
}
```

### Key Concepts

**String Parsing:**
- `lines()` iterates over lines
- `split_once()` splits on first occurrence
- `find()` locates substring, returns Option

**File Operations:**
- `fs::read_to_string()` reads entire file
- `fs::write()` writes entire file
- Always backup before modifying user config!

---

## Part 3: System Integration (Systemd)

### Why Systemd?

Instead of directly mounting/unmounting (which requires root), we create systemd user services. Benefits:
- No root required
- Services can restart on failure
- Status tracking via systemd
- Integration with desktop session

### Systemd Manager Structure

```rust
// src/services/mod.rs

use anyhow::{anyhow, Result};
use std::process::Command;

/// Manage rclone mount systemd services
pub struct SystemdManager;

impl SystemdManager {
    /// Generate a systemd service name for a mount
    /// Example: "rclone-mount-gdrive-home-user-gdrive.service"
    pub fn service_name(remote: &str, mount_point: &str) -> String {
        let sanitized = mount_point
            .replace('/', "-")
            .replace("~", "home")
            .trim_matches('-')
            .to_string();
        format!("rclone-mount-{}-{}.service", remote, sanitized)
    }

    /// Generate complete systemd service file content
    pub fn generate_service(
        remote: &str,
        mount_point: &str,
        mount_options: &str,
    ) -> String {
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
}
```

**INI Format for Systemd:**
- `[Unit]` section describes the service
- `[Service]` section contains execution details
- `[Install]` section describes how to enable/disable
- `Type=notify` means rclone handles readiness signaling

### Systemd Commands via subprocess

```rust
impl SystemdManager {
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

    /// Check if mount is currently running
    pub fn is_mounted(service_name: &str) -> Result<bool> {
        let output = Command::new("systemctl")
            .args(&["--user", "is-active", service_name])
            .output()?;

        Ok(output.status.success())
    }

    /// Get service status details
    pub fn get_status(service_name: &str) -> Result<String> {
        let output = Command::new("systemctl")
            .args(&["--user", "status", service_name])
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Enable service to start on login
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

    /// Disable service from starting on login
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

    /// Reload systemd daemon (required after creating new services)
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
```

### Key Concepts

**Process Spawning:**
- `Command::new()` starts building a command
- `.args()` adds arguments
- `.output()` runs and captures stdout/stderr
- Check `status.success()` to verify execution

**Error Details:**
- stderr contains error messages
- Always convert to string for user display
- `?` operator automatically propagates errors

---

## Part 4: GTK4 UI Fundamentals

### GTK4 vs GTK3 vs Other Toolkits

| Aspect | GTK4 | Qt | Flutter |
|--------|------|----|---------| 
| Learning Curve | Medium | Steep | Medium |
| Rust Support | Excellent | Good | Emerging |
| Linux Native | Excellent | Good | Emulated |
| Performance | Excellent | Excellent | Good |
| Best For | GNOME Apps | Enterprise | Cross-platform |

GTK4 is ideal because:
- Modern API design (2020+)
- Excellent Rust bindings
- Native GNOME integration
- Small binary size

### Hello World GTK4 Application

```rust
// src/main.rs

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
use std::io;

const APP_ID: &str = "com.github.rclone-config-manager";

fn main() -> glib::ExitCode {
    // Create application
    let app = Application::new(Some(APP_ID), gio::ApplicationFlags::FLAGS_NONE);

    // Handle startup
    app.connect_startup(|_| {
        println!("Application startup");
    });

    // Handle activation (window shown)
    app.connect_activate(|app| {
        build_ui(app);
    });

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    // Create main window
    let window = ApplicationWindow::new(app);
    window.set_title(Some("RClone Config Manager"));
    window.set_default_size(800, 600);

    // Add a label as child widget
    let label = gtk4::Label::new(Some("Hello, Rust GTK4!"));
    window.set_child(Some(&label));

    // Show the window
    window.present();
}
```

**Key Concepts:**

1. **Application Struct**: Manages app lifecycle
2. **Windows**: Top-level containers that hold UI
3. **Widgets**: Individual UI elements (buttons, labels, etc.)
4. **Signals**: Connect to events (click, text-changed, etc.)
5. **Hierarchy**: Widgets contain other widgets (tree structure)

### Widget Hierarchy and Containers

```rust
use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation};

fn build_ui(app: &Application) {
    let window = ApplicationWindow::new(app);
    window.set_title(Some("Widget Hierarchy Example"));
    window.set_default_size(400, 300);

    // Create main vertical box
    let main_box = Box::new(Orientation::Vertical, 12);
    main_box.set_margin_top(12);
    main_box.set_margin_bottom(12);
    main_box.set_margin_start(12);
    main_box.set_margin_end(12);

    // Create a header section
    let header_box = Box::new(Orientation::Horizontal, 12);
    let title = Label::new(Some("My Application"));
    title.add_css_class("title-1"); // CSS class for styling
    header_box.append(&title);

    // Add stretch fill (pushes button to the right)
    header_box.set_hexpand(true);
    
    let add_button = Button::with_label("Add Item");
    header_box.append(&add_button);

    // Append header to main box
    main_box.append(&header_box);

    // Add content section
    let content_label = Label::new(Some("Content goes here"));
    content_label.set_hexpand(true);
    content_label.set_vexpand(true);
    main_box.append(&content_label);

    // Set window content
    window.set_child(Some(&main_box));
    window.present();
}
```

**Visual Layout:**
```
┌─────────────────────────────────┐
│ My Application        [Add Item] │  <- header_box (Horizontal)
├─────────────────────────────────┤
│                                 │
│  Content goes here              │  <- content_label (expands)
│                                 │
└─────────────────────────────────┘
```

### Styling with CSS

```rust
use gtk4::{CssProvider, gdk};

fn apply_css(app: &Application) {
    let css_provider = CssProvider::new();
    css_provider.load_from_data(r#"
        .title-1 {
            font-size: 28px;
            font-weight: bold;
        }

        .title-2 {
            font-size: 20px;
            font-weight: bold;
        }

        .monospace {
            font-family: monospace;
            font-size: 11px;
        }

        button {
            padding: 6px 12px;
            margin: 4px;
        }

        button.destructive-action {
            background-color: #e74c3c;
            color: white;
        }

        button.suggested-action {
            background-color: #27ae60;
            color: white;
        }
    "#);

    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
```

**Using CSS:**
- Define styles as strings
- Add CSS classes to widgets with `add_css_class()`
- Apply to display for global effect
- Override inline properties

### Handling Signals (Events)

```rust
use gtk4::prelude::*;
use gtk4::Button;

fn button_example() {
    let button = Button::with_label("Click Me!");

    // Connect to clicked signal
    button.connect_clicked(|btn| {
        println!("Button clicked!");
        btn.set_label("Clicked!");
    });

    // Multiple signals on same widget
    button.connect_focus_in(|_| {
        println!("Button has focus");
        glib::signal::Inhibit(false) // Don't stop propagation
    });
}
```

**Key Concepts:**
- `connect_*` methods register signal handlers
- Closures capture surrounding variables
- Return `Inhibit(true)` to stop signal propagation
- Many widgets have different signals

---

## Part 5: Building Dialogs & Widgets

### Dialog Pattern

```rust
// src/ui/dialogs.rs

use gtk4::prelude::*;
use gtk4::{Dialog, Entry, Label, Box, Orientation, ResponseType};
use crate::models::RemoteConfig;

/// Dialog for adding/editing a remote
pub struct AddRemoteDialog {
    dialog: Dialog,
    name_entry: Entry,
    service_combo: gtk4::ComboBoxText,
}

impl AddRemoteDialog {
    /// Create dialog (not shown yet)
    pub fn new(parent_window: &impl IsA<gtk4::Window>) -> Self {
        let dialog = Dialog::new();
        dialog.set_title(Some("Add Remote"));
        dialog.set_transient_for(Some(parent_window));
        dialog.set_modal(true); // Block interaction with parent
        dialog.set_default_size(500, 400);

        // Build content
        let content_area = dialog.content_area();
        let main_box = Box::new(Orientation::Vertical, 12);
        main_box.set_margin_top(12);
        main_box.set_margin_bottom(12);
        main_box.set_margin_start(12);
        main_box.set_margin_end(12);

        // Name field
        let name_label = Label::new(Some("Remote Name:"));
        name_label.set_halign(gtk4::Align::Start);
        main_box.append(&name_label);

        let name_entry = Entry::new();
        name_entry.set_placeholder_text(Some("e.g., my-google-drive"));
        main_box.append(&name_entry);

        // Service selection
        let service_label = Label::new(Some("Cloud Service:"));
        service_label.set_halign(gtk4::Align::Start);
        main_box.append(&service_label);

        let service_combo = gtk4::ComboBoxText::new();
        service_combo.append_text("Google Drive");
        service_combo.append_text("OneDrive");
        service_combo.append_text("Dropbox");
        service_combo.set_active(Some(0)); // Select first item by default
        main_box.append(&service_combo);

        content_area.append(&main_box);

        // Add buttons
        dialog.add_button("Cancel", ResponseType::Cancel as i32);
        dialog.add_button("Save", ResponseType::Accept as i32);
        dialog.set_default_response(ResponseType::Accept as i32);

        Self {
            dialog,
            name_entry,
            service_combo,
        }
    }

    /// Run dialog and get result
    pub fn run(&self) -> Option<RemoteConfig> {
        let response = self.dialog.run();

        // Check which button was clicked
        if response == ResponseType::Accept as i32 {
            let name = self.name_entry.text().to_string();
            let service_idx = self.service_combo.active()? as usize;
            
            let service = match service_idx {
                0 => crate::models::CloudService::GoogleDrive,
                1 => crate::models::CloudService::OneDrive,
                2 => crate::models::CloudService::Dropbox,
                _ => crate::models::CloudService::GoogleDrive,
            };

            Some(RemoteConfig::new(name, service))
        } else {
            None
        }
    }
}

// Usage:
fn show_add_remote_dialog(window: &ApplicationWindow) {
    let dialog = AddRemoteDialog::new(window);
    
    if let Some(remote) = dialog.run() {
        println!("User created remote: {}", remote.name);
        // TODO: Save to config
    }
}
```

**Dialog Pattern:**
1. Create struct holding dialog and input widgets
2. `new()` builds UI without showing
3. `run()` displays dialog and waits for response
4. Construct and return model from user input

### Reusable List Row Widget

```rust
// src/ui/widgets.rs

use gtk4::prelude::*;
use gtk4::{Box, Button, Label, ListBoxRow, Orientation};
use crate::models::RemoteConfig;

/// Create a list row for displaying a remote
pub fn create_remote_row(remote: &RemoteConfig) -> ListBoxRow {
    let row = ListBoxRow::new();
    
    // Horizontal box for left-to-right layout
    let hbox = Box::new(Orientation::Horizontal, 12);
    hbox.set_margin_top(6);
    hbox.set_margin_bottom(6);
    hbox.set_margin_start(6);
    hbox.set_margin_end(6);

    // Icon + service type
    let icon_label = Label::new(Some(remote.service.icon_char()));
    icon_label.add_css_class("title-4");
    hbox.append(&icon_label);

    // Vertical box for name and service
    let info_box = Box::new(Orientation::Vertical, 2);
    
    let name_label = Label::new(Some(&remote.name));
    name_label.set_halign(gtk4::Align::Start);
    name_label.add_css_class("title-4");
    info_box.append(&name_label);

    let service_label = Label::new(Some(remote.service.display_name()));
    service_label.set_halign(gtk4::Align::Start);
    service_label.add_css_class("dim-label");
    info_box.append(&service_label);

    hbox.append(&info_box);
    hbox.set_hexpand(true); // Take available space

    // Action buttons
    let edit_btn = Button::with_label("Edit");
    edit_btn.add_css_class("suggested-action");
    hbox.append(&edit_btn);

    let delete_btn = Button::with_label("Delete");
    delete_btn.add_css_class("destructive-action");
    hbox.append(&delete_btn);

    row.set_child(Some(&hbox));
    row
}
```

**Visual Result:**
```
┌──────────────────────────────────────────┐
│ 🔵 My Google Drive        [Edit] [Delete] │
│     Google Drive                          │
└──────────────────────────────────────────┘
```

### Message Dialogs

```rust
use gtk4::{MessageDialog, MessageType, ButtonsType};

/// Show error message to user
pub fn show_error_dialog(
    parent: &impl IsA<gtk4::Window>,
    title: &str,
    message: &str,
) {
    let dialog = MessageDialog::new(
        Some(parent),
        gtk4::DialogFlags::MODAL,
        MessageType::Error,
        ButtonsType::Ok,
        title,
    );
    dialog.set_secondary_text(Some(message));
    
    // Show and close when user clicks OK
    dialog.run_async(|dialog, _| {
        dialog.close();
    });
}

/// Show confirmation dialog (blocking)
pub fn show_confirm_dialog(
    parent: &impl IsA<gtk4::Window>,
    title: &str,
    message: &str,
) -> bool {
    let dialog = MessageDialog::new(
        Some(parent),
        gtk4::DialogFlags::MODAL,
        MessageType::Question,
        ButtonsType::YesNo,
        title,
    );
    dialog.set_secondary_text(Some(message));
    
    let response = dialog.run();
    dialog.close();
    
    response == ResponseType::Yes as i32
}
```

---

## Part 6: Event Handling & State Management

### Shared State with Arc + Mutex

```rust
// src/main.rs

use std::sync::{Arc, Mutex};
use crate::config::RcloneConfigManager;

fn main() -> glib::ExitCode {
    let app = Application::new(Some(APP_ID), gio::ApplicationFlags::FLAGS_NONE);

    // Create shared state
    let config_manager = Arc::new(Mutex::new(
        RcloneConfigManager::new()
            .expect("Failed to initialize config manager")
    ));

    let config_manager_clone = config_manager.clone(); // For use in closure

    app.connect_activate(move |app| {
        build_ui(app, config_manager_clone.clone());
    });

    app.run()
}

fn build_ui(app: &Application, config_manager: Arc<Mutex<RcloneConfigManager>>) {
    let window = ApplicationWindow::new(app);
    
    // Access shared state
    {
        let cm = config_manager.lock().unwrap();
        let remotes = cm.parse_remotes().unwrap_or_default();
        println!("Loaded {} remotes", remotes.len());
    } // Lock released here
    
    window.present();
}
```

**Why Arc + Mutex?**
- `Arc` = Atomic Reference Counted (shared ownership)
- `Mutex` = Mutual exclusion (safe concurrent access)
- `.lock()` gets exclusive access
- Lock is automatically released when `MutexGuard` drops

### Button Click Handlers

```rust
use std::sync::{Arc, Mutex};
use gtk4::Button;

fn setup_button_handlers(
    add_button: &Button,
    config_manager: Arc<Mutex<RcloneConfigManager>>,
    window: &ApplicationWindow,
) {
    let config_manager_clone = config_manager.clone();
    let window_clone = window.clone();

    add_button.connect_clicked(move |_| {
        // Clone again for use in dialog closure
        let config_manager = config_manager_clone.clone();
        let window = window_clone.clone();

        // Show dialog
        let dialog = AddRemoteDialog::new(&window);
        if let Some(remote) = dialog.run() {
            // Save remote using shared config manager
            match config_manager.lock() {
                Ok(cm) => {
                    if let Err(e) = cm.write_remote(&remote) {
                        show_error_dialog(&window, "Error", &e.to_string());
                    }
                }
                Err(e) => {
                    show_error_dialog(&window, "Error", &format!("Lock failed: {}", e));
                }
            }
        }
    });
}
```

### Refresh UI from Data

```rust
use gtk4::{ListBox, prelude::*};

fn refresh_remotes_list(
    list_box: &ListBox,
    config_manager: Arc<Mutex<RcloneConfigManager>>,
    window: &ApplicationWindow,
) {
    // Lock config manager
    match config_manager.lock() {
        Ok(cm) => {
            // Parse remotes from config file
            match cm.parse_remotes() {
                Ok(remotes) => {
                    // Clear existing rows
                    while let Some(child) = list_box.first_child() {
                        list_box.remove(&child);
                    }

                    // Add rows for each remote
                    for remote in remotes {
                        let row = create_remote_row(&remote);
                        list_box.append(&row);
                    }
                }
                Err(e) => {
                    show_error_dialog(
                        window,
                        "Error Loading Remotes",
                        &e.to_string(),
                    );
                }
            }
        }
        Err(e) => {
            show_error_dialog(window, "Error", &format!("Lock failed: {}", e));
        }
    }
}
```

---

## Part 7: System Tray Integration

### System Tray with tray-icon Crate

```rust
// For system tray integration
cargo add tray-icon muda

// Cargo.toml
[dependencies]
tray-icon = "0.0.12"
muda = "0.12"
winit = "0.29"  # Event loop
```

### Building a System Tray App

```rust
// src/main.rs (system tray version)

use tray_icon::TrayIconBuilder;
use muda::Menu;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create event loop
    let event_loop = winit::event_loop::EventLoop::new()?;

    // Create tray menu
    let menu = Menu::new();
    let menu_items = vec![
        muda::MenuItem::String(muda::PredefinedMenuItem::show(None)),
        muda::MenuItem::Separator,
        muda::MenuItem::Check(CheckMenuItem::new("Auto-mount", true, true, None)),
        muda::MenuItem::Separator,
        muda::MenuItem::String(muda::PredefinedMenuItem::quit(None)),
    ];

    for item in menu_items {
        menu.append_items(&[item])?;
    }

    // Create tray icon
    let icon = load_icon();
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_icon(icon)
        .build()?;

    // Event loop
    event_loop.run(move |event, _, control_flow| {
        match event {
            winit::event::Event::UserEvent(CustomEvent::TrayIconEvent(event)) => {
                match event {
                    TrayIconEvent::DoubleClick { .. } => {
                        // Show main window
                    }
                    TrayIconEvent::MenuItemClick { id } => {
                        // Handle menu clicks
                    }
                    _ => {}
                }
            }
            winit::event::Event::MainEventsCleared => {
                // Update tray status
            }
            _ => {}
        }
    })?;

    Ok(())
}

fn load_icon() -> tray_icon::Icon {
    // Load icon from embedded bytes or file
    let rgba = vec![/* RGBA pixel data */];
    tray_icon::Icon::from_rgba(rgba, 32, 32).unwrap()
}
```

### Status Updates

```rust
use std::time::{Duration, Instant};

fn update_tray_status(
    tray_icon: &tray_icon::TrayIcon,
    config_manager: Arc<Mutex<RcloneConfigManager>>,
) {
    // Check if mounts are active
    match SystemdManager::list_services() {
        Ok(services) => {
            let mut all_mounted = true;
            for service in &services {
                if !SystemdManager::is_mounted(service).unwrap_or(false) {
                    all_mounted = false;
                    break;
                }
            }

            // Update icon based on status
            let icon = if all_mounted {
                load_icon_green()  // All mounted
            } else if services.is_empty() {
                load_icon_gray()   // No mounts
            } else {
                load_icon_yellow() // Partial
            };

            tray_icon.set_icon(Some(icon)).ok();
        }
        Err(_) => {
            tray_icon.set_icon(Some(load_icon_red())).ok();
        }
    }
}
```

---

## Part 8: Packaging & Distribution

### Debian Package Structure

```
debian/
├── control              # Package metadata
├── rules                # Build instructions
├── changelog            # Release history
├── copyright            # License info
├── postinst            # Post-install script
├── postrm              # Post-remove script
├── compat              # Debian compat level
├── source/
│   └── format          # Source package format
├── tests/
│   ├── control         # Test metadata
│   └── basic-functionality
└── install             # File installation
```

### Control File

```
Source: rclone-config-manager
Section: utils
Priority: optional
Maintainer: Your Name <you@example.com>
Homepage: https://github.com/yourusername/rclone-config-manager
Standards-Version: 4.7.0
Build-Depends: cargo (>= 1.70), rustc (>= 1.70), libgtk-4-dev, libadwaita-1-dev
Rules-Requires-Root: no

Package: rclone-config-manager
Architecture: amd64 arm64
Depends: ${misc:Depends}, libgtk-4-1, libadwaita-1, systemd, rclone
Description: GTK4 GUI for managing rclone mounts
 Manage rclone remote configurations and mounts with a modern
 GNOME-style GTK4 interface.
 .
 Features:
  - Multi-cloud service support
  - OAuth authentication
  - Systemd integration
```

### Debian Rules File

```make
#!/usr/bin/make -f

export CARGO_HOME=$(CURDIR)/.cargo
export PATH:=$(HOME)/.cargo/bin:$(PATH)

%:
	dh $@

# Build Rust application
override_dh_auto_build:
	cargo build --release --locked --verbose

# Run tests
override_dh_auto_test:
	cargo test --release --locked --verbose || true

# Install binaries
override_dh_auto_install:
	dh_auto_install

# Clean build artifacts
override_dh_auto_clean:
	cargo clean || true
	dh_auto_clean

# Strip binaries
override_dh_strip:
	dh_strip --no-automatic-dbgsym
```

### Building the Package

```bash
# Build locally
debian/rules binary

# Or use dpkg-buildpackage
dpkg-buildpackage -b -uc -us

# Create source package
dpkg-buildpackage -S -uc -us

# Install locally
sudo dpkg -i ../rclone-config-manager_0.1.0-1_amd64.deb
```

### Desktop Entry

```ini
# /usr/share/applications/rclone-config-manager.desktop

[Desktop Entry]
Type=Application
Name=RClone Config Manager
Comment=Manage rclone mounts and authentication
Icon=system-file-manager
Exec=rclone-config-manager
Categories=System;Utility;FileManager;
Terminal=false
StartupNotify=true
```

---

## Appendix: Common Patterns & Troubleshooting

### Pattern 1: Safe Unwrapping

```rust
// ❌ Bad - panics if None
let value = option.unwrap();

// ❌ Bad - unwraps Result, loses error info
let value = result.unwrap();

// ✅ Good - handle both cases
match option {
    Some(v) => println!("Value: {}", v),
    None => println!("No value"),
}

// ✅ Good - propagate error with ?
let value = fallible_operation()?;

// ✅ Good - provide default
let value = option.unwrap_or(default_value);

// ✅ Good - log and handle
match result {
    Ok(v) => { /* use v */ }
    Err(e) => {
        tracing::error!("Operation failed: {}", e);
        // Recovery logic
    }
}
```

### Pattern 2: Logging

```rust
use tracing::{info, debug, warn, error};

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("Application started");
    debug!("Debug details");
    warn!("Warning message");
    error!("Error occurred");
}

// View logs:
// journalctl --user-unit rclone-mount-tray -f
```

### Pattern 3: Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_type_parsing() {
        let service = CloudService::GoogleDrive;
        assert_eq!(service.as_str(), "drive");
    }

    #[test]
    fn test_mount_config_creation() {
        let mount = MountConfig::new(
            "test".to_string(),
            "gdrive".to_string(),
            "/mnt/gdrive".to_string(),
        );
        assert_eq!(mount.name, "test");
        assert!(mount.enabled);
    }

    #[test]
    #[should_panic]
    fn test_panic() {
        panic!("This test verifies panic behavior");
    }
}
```

Run tests:
```bash
cargo test
cargo test --lib
cargo test integration_tests
```

### Common GTK4 Errors

**Error: Cannot find GTK4 libraries**
```bash
# Solution: Install development packages
sudo apt-get install libgtk-4-dev libadwaita-1-dev

# Set PKG_CONFIG_PATH if needed
export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig:$PKG_CONFIG_PATH
```

**Error: "thread panicked: called `Result::unwrap()` on an `Err` value"**
```rust
// Problem: unwrap() panics on error
let config = RcloneConfigManager::new().unwrap();

// Solution: handle error properly
match RcloneConfigManager::new() {
    Ok(cm) => { /* use cm */ }
    Err(e) => {
        eprintln!("Failed to initialize: {}", e);
        std::process::exit(1);
    }
}
```

**Error: Cannot move value into closure**
```rust
// Problem: ownership issue
let value = String::from("hello");
button.connect_clicked(|| {
    println!("{}", value); // value moved already
});

// Solution: clone for closure
let value_clone = value.clone();
button.connect_clicked(move || {
    println!("{}", value_clone);
});
```

### Performance Tips

**1. Minimize Lock Contention**
```rust
// ❌ Bad - holds lock during long operation
{
    let cm = config_manager.lock().unwrap();
    let remotes = cm.parse_remotes()?;
    perform_expensive_operation(&remotes);
}

// ✅ Good - release lock early
let remotes = {
    let cm = config_manager.lock().unwrap();
    cm.parse_remotes()?
}; // Lock released
perform_expensive_operation(&remotes);
```

**2. Use Async for I/O**
```rust
use tokio::fs;

// ❌ Bad - blocks thread
let config = std::fs::read_to_string("config.toml")?;

// ✅ Good - non-blocking
let config = tokio::fs::read_to_string("config.toml").await?;
```

**3. Profile Your Application**
```bash
# Build with profiling info
cargo build --release

# Run with perf
perf record ./target/release/rclone-config-manager
perf report

# Check binary size
ls -lh target/release/rclone-config-manager
```

---

## Final Project Structure

```
rclone-config-manager/
├── Cargo.toml                   # Project config
├── Cargo.lock                   # Dependency lock
├── README.md                    # Documentation
│
├── src/
│   ├── main.rs                  # Entry point (51 lines)
│   ├── models/
│   │   └── mod.rs              # Data models (186 lines)
│   ├── config/
│   │   └── mod.rs              # Config mgmt (183 lines)
│   ├── auth/
│   │   └── mod.rs              # Authentication (160 lines)
│   ├── services/
│   │   └── mod.rs              # Systemd integration (157 lines)
│   └── ui/
│       ├── mod.rs              # Main window (345 lines)
│       ├── dialogs.rs          # Dialogs (359 lines)
│       └── widgets.rs          # Widgets (168 lines)
│
├── assets/
│   └── style.css               # UI styling
│
├── debian/
│   ├── control                 # Package metadata
│   ├── rules                   # Build rules
│   ├── changelog               # Release notes
│   ├── install                 # File installation
│   └── ...
│
└── .github/
    └── workflows/
        ├── ci.yml              # CI/CD pipeline
        └── release.yml         # Release automation
```

**Total Code:** ~1,609 lines of Rust
**Binary Size:** ~10-15 MB (release)
**Memory Usage:** 50-100 MB at runtime

---

## Key Takeaways

### Rust Programming
✅ Type safety catches errors at compile time
✅ Ownership system prevents memory bugs
✅ Pattern matching forces exhaustive handling
✅ Error handling with Result types
✅ Excellent async/await support

### GUI Development
✅ GTK4 provides modern, native Linux interface
✅ Separation of concerns: UI ≠ Business Logic
✅ Signals/events drive interaction
✅ Containers and layout managers handle positioning
✅ CSS styling for professional appearance

### System Integration
✅ Systemd user services avoid elevated privileges
✅ Config file parsing for user data
✅ Process management via subprocess
✅ Proper error handling and logging
✅ Integration with desktop environment

### Professional Development
✅ Debian packaging for distribution
✅ Comprehensive error handling
✅ Logging for debugging
✅ Testing for reliability
✅ Version control best practices

---

## Next Steps

1. **Extend Functionality**: Add more cloud services
2. **Improve UI**: Add status indicators, progress bars
3. **Add Features**: OAuth flows, token refresh, caching
4. **Optimize**: Profile and optimize hot paths
5. **Test**: Write comprehensive unit and integration tests
6. **Document**: Create user manual and API docs
7. **Deploy**: Set up CI/CD, automatic builds
8. **Maintain**: Update dependencies, fix bugs

---

## Resources

- [GTK4 Rust Documentation](https://gtk-rs.org/gtk4-rs/stable/latest/)
- [Tokio Async Runtime](https://tokio.rs/)
- [Debian Packaging Guide](https://www.debian.org/doc/debian-policy/)
- [rclone Documentation](https://rclone.org/)
- [Rust Book](https://doc.rust-lang.org/book/)

---

**Happy Rust coding! 🦀**
