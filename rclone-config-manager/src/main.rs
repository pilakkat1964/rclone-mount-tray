// RClone Config Manager - Application Entry Point

mod models;
mod config;
mod auth;
mod services;

#[cfg(feature = "gui")]
mod ui;

use config::RcloneConfigManager;
use services::SystemdManager;

#[cfg(feature = "gui")]
use ui::AppWindow;

#[cfg(feature = "gui")]
use gtk4::prelude::*;
#[cfg(feature = "gui")]
use gtk4::{Application, gio};

const APP_ID: &str = "com.github.pilakkat1964.rclone-config-manager";

#[cfg(feature = "gui")]
fn main() -> glib::ExitCode {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    // Create GTK application
    let app = Application::new(Some(APP_ID), gio::ApplicationFlags::FLAGS_NONE);

    // Connect signals
    app.connect_startup(|app| {
        // Load CSS
        let css_provider = gtk4::CssProvider::new();
        css_provider.load_from_data(include_str!("../assets/style.css"));
        
        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &css_provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        tracing::info!("Application startup");
    });

    app.connect_activate(|app| {
        let app_window = AppWindow::new(app);
        app_window.present();

        tracing::info!("Application activated");
    });

    // Run the application
    app.run()
}

#[cfg(not(feature = "gui"))]
fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    eprintln!("Error: This application requires the 'gui' feature to run.");
    eprintln!();
    eprintln!("The RClone Config Manager is a GUI-only application that requires GTK4.");
    eprintln!();
    eprintln!("To build with GUI support, ensure GTK4 system libraries are installed:");
    eprintln!("  Debian/Ubuntu: sudo apt-get install libgtk-4-dev libadwaita-1-dev");
    eprintln!("  Fedora/RHEL:   sudo dnf install gtk4-devel libadwaita-devel");
    eprintln!("  Arch:          sudo pacman -S gtk4 libadwaita");
    eprintln!();
    eprintln!("Then build with:");
    eprintln!("  cargo build --features gui --release");
    eprintln!();
    eprintln!("For more information, see GTK4_SETUP.md");
    std::process::exit(1);
}
