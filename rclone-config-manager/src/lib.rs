// RClone Config Manager - Library for non-GUI components

pub mod models;
pub mod config;
pub mod auth;
pub mod services;

#[cfg(feature = "gui")]
pub mod ui;

// Re-export commonly used types
pub use models::*;
pub use config::RcloneConfigManager;
pub use auth::*;
pub use services::SystemdManager;
