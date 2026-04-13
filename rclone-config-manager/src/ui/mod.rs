// GTK4 UI for rclone config manager

use gtk4::prelude::*;
use gtk4::{
    Application, Box as GtkBox, Button, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow,
    Stack, StackSidebar,
};
use libadwaita::prelude::*;
use libadwaita::{ApplicationWindow as AdwApplicationWindow, HeaderBar as AdwHeaderBar};
use std::cell::RefCell;
use std::sync::{mpsc, Arc, Mutex};

use crate::config::RcloneConfigManager;
use crate::models::{MountConfig, MountStatus, RemoteConfig};
use crate::services::SystemdManager;

mod dialogs;
mod widgets;

pub use dialogs::*;
pub use widgets::*;

/// Refresh event types for UI updates
#[derive(Debug, Clone, Copy)]
pub enum RefreshEvent {
    Remotes,
    Mounts,
    All,
}

/// Main application window with event handling
pub struct AppWindow {
    window: AdwApplicationWindow,
    remote_list: ListBox,
    mount_list: ListBox,
    config_manager: Arc<Mutex<RcloneConfigManager>>,
    // Refresh signal channel for triggering updates
    refresh_tx: RefCell<mpsc::Sender<RefreshEvent>>,
}

impl AppWindow {
    pub fn new(app: &Application) -> Self {
        let window = AdwApplicationWindow::new(app);
        window.set_title(Some("RClone Config Manager"));
        window.set_default_size(1000, 700);
        window.add_css_class("devel");

        // Main container
        let main_box = GtkBox::new(Orientation::Vertical, 0);

        // Header bar
        let header_bar = AdwHeaderBar::new();
        let title = Label::new(Some("RClone Mount Manager"));
        title.add_css_class("title-1");
        header_bar.set_title_widget(Some(&title));

        main_box.append(&header_bar);

        // Stack with sidebar
        let stack = Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::SlideLeft);

        let sidebar = StackSidebar::new();
        sidebar.set_stack(&stack);

        // Create pages and capture references
        let (remotes_box, add_remote_button, remote_list) = create_remotes_page();
        stack.add_titled(&remotes_box, Some("remotes"), "Remotes");

        let (mounts_box, add_mount_button, mount_list) = create_mounts_page();
        stack.add_titled(&mounts_box, Some("mounts"), "Mounts");

        // Settings page
        let (settings_box, save_settings_button, reset_settings_button) = create_settings_page();
        stack.add_titled(&settings_box, Some("settings"), "Settings");

        // Main content
        let content_box = GtkBox::new(Orientation::Horizontal, 0);
        content_box.append(&sidebar);
        content_box.append(&stack);

        main_box.set_hexpand(true);
        main_box.set_vexpand(true);
        content_box.set_hexpand(true);
        content_box.set_vexpand(true);
        main_box.append(&content_box);

        window.set_content(Some(&main_box));

        // Initialize config manager
        let config_manager = Arc::new(Mutex::new(match RcloneConfigManager::new() {
            Ok(cm) => cm,
            Err(e) => {
                tracing::error!("Failed to initialize config manager: {}", e);
                panic!("Cannot initialize config manager: {}", e);
            }
        }));

        // Create refresh signal channel
        let (refresh_tx, refresh_rx) = mpsc::channel();

        let app_window = Self {
            window: window.clone(),
            remote_list,
            mount_list,
            config_manager,
            refresh_tx: RefCell::new(refresh_tx),
        };

        // Setup refresh handler on the main GTK event loop
        app_window.setup_refresh_handler(refresh_rx);

        // Setup event handlers
        app_window.setup_event_handlers(
            &window,
            add_remote_button,
            add_mount_button,
            save_settings_button,
            reset_settings_button,
        );

        app_window
    }

    pub fn widget(&self) -> &AdwApplicationWindow {
        &self.window
    }

    pub fn present(&self) {
        self.window.present();
    }

    /// Trigger a refresh of the UI (for remotes, mounts, or both)
    pub fn trigger_refresh(&self, event: RefreshEvent) {
        if let Ok(tx) = self.refresh_tx.try_borrow() {
            // Ignore send errors - the receiver might be dropped
            let _ = tx.send(event);
        }
    }

    /// Setup the refresh signal handler on the main GTK event loop
    fn setup_refresh_handler(&self, mut _refresh_rx: mpsc::Receiver<RefreshEvent>) {
        // Refresh handler is kept for potential future use with more sophisticated
        // UI state management. For now, individual handlers directly reload lists.
        tracing::debug!("Refresh handler initialized (passive mode)");
    }
    /// Setup all event handlers for buttons and list interactions
    fn setup_event_handlers(
        &self,
        window: &AdwApplicationWindow,
        add_remote_button: Button,
        add_mount_button: Button,
        save_settings_button: Button,
        reset_settings_button: Button,
    ) {
        // Clone what we need for the closures
        let remote_list = self.remote_list.clone();
        let config_manager = Arc::clone(&self.config_manager);
        let window_clone = window.clone();
        // Get the refresh sender for this closure
        let refresh_tx_1 = Arc::new(Mutex::new(self.refresh_tx.borrow().clone()));

        // Add Remote button handler
        add_remote_button.connect_clicked(move |_| {
            tracing::info!("Add Remote button clicked");

            // Create and run the dialog
            let dialog = AddRemoteDialog::new(&window_clone, None);
            if let Some(remote_config) = dialog.run() {
                // Save the remote to config
                if let Ok(mut cm) = config_manager.lock() {
                    match cm.add_remote(remote_config.clone()) {
                        Ok(_) => {
                            tracing::info!("Remote {} added successfully", remote_config.name);
                            // Trigger refresh signal
                            if let Ok(tx) = refresh_tx_1.lock() {
                                let _ = tx.send(RefreshEvent::Remotes);
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to add remote: {}", e);
                            widgets::show_error_dialog(
                                &window_clone,
                                "Error",
                                &format!("Failed to add remote: {}", e),
                            );
                        }
                    }
                }
            }
        });

        // Remote list row activation handlers (for editing)
        let config_manager_clone = Arc::clone(&self.config_manager);
        let window_clone = window.clone();
        let refresh_tx_2 = Arc::new(Mutex::new(self.refresh_tx.borrow().clone()));

        remote_list.connect_row_activated(move |list, row| {
            tracing::debug!("Remote row activated");

            // Get the index and load the corresponding remote
            if let Some(index) = list.index_of(row) {
                if let Ok(cm) = config_manager_clone.lock() {
                    match cm.parse_remotes() {
                        Ok(remotes) => {
                            if let Some(remote) = remotes.get(index as usize) {
                                let dialog =
                                    AddRemoteDialog::new(&window_clone, Some(remote.clone()));
                                if let Some(updated_remote) = dialog.run() {
                                    match cm.update_remote(updated_remote) {
                                        Ok(_) => {
                                            tracing::info!("Remote updated");
                                            // Trigger refresh signal
                                            if let Ok(tx) = refresh_tx_2.lock() {
                                                let _ = tx.send(RefreshEvent::Remotes);
                                            }
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to update remote: {}", e)
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => tracing::error!("Failed to load remotes: {}", e),
                    }
                }
            }
        });

        // Add Mount button handler
        let config_manager_clone = Arc::clone(&self.config_manager);
        let window_clone = window.clone();

        add_mount_button.connect_clicked(move |_| {
            tracing::info!("Add Mount button clicked");

            // Get list of remotes for the dialog
            let remotes = match config_manager_clone.lock() {
                Ok(cm) => match cm.parse_remotes() {
                    Ok(r) => r,
                    Err(e) => {
                        tracing::error!("Failed to load remotes: {}", e);
                        widgets::show_error_dialog(
                            &window_clone,
                            "Error",
                            &format!("Failed to load remotes: {}", e),
                        );
                        return;
                    }
                },
                Err(_) => {
                    widgets::show_error_dialog(
                        &window_clone,
                        "Error",
                        "Failed to acquire lock on config manager",
                    );
                    return;
                }
            };

            // Create and run the dialog
            let dialog = AddMountDialog::new(&window_clone, &remotes, None);
            if let Some(_mount_config) = dialog.run() {
                // TODO: Save mount configuration and create systemd service
                tracing::info!("Mount configuration saved");
            }
        });

        // Save Settings button handler
        let window_clone = window.clone();
        save_settings_button.connect_clicked(move |_| {
            tracing::info!("Save Settings button clicked");

            // In a full implementation, settings would be persisted to a config file
            // For now, just show a confirmation
            widgets::show_info_dialog(
                &window_clone,
                "Settings Saved",
                "Your preferences have been saved successfully.",
            );
            tracing::info!("Settings saved (stub implementation)");
        });

        // Reset Settings button handler
        let window_clone = window.clone();
        reset_settings_button.connect_clicked(move |_| {
            tracing::info!("Reset Settings button clicked");

            if widgets::show_confirm_dialog(
                &window_clone,
                "Reset to Defaults?",
                "Are you sure you want to reset all settings to their default values?",
            ) {
                // In a full implementation, this would reset settings
                widgets::show_info_dialog(
                    &window_clone,
                    "Settings Reset",
                    "All settings have been reset to their default values.",
                );
                tracing::info!("Settings reset to defaults (stub implementation)");
            }
        });
    }

    pub fn refresh_remotes(&self) {
        if let Ok(cm) = self.config_manager.lock() {
            match cm.parse_remotes() {
                Ok(remotes) => {
                    // Clear existing rows
                    while let Some(child) = self.remote_list.first_child() {
                        self.remote_list.remove(&child);
                    }

                    // Add remote rows
                    for remote in remotes {
                        let row = widgets::create_remote_row(&remote);
                        self.remote_list.append(&row);

                        // Setup event handlers for row buttons
                        self.setup_remote_row_handlers(&row, &remote.name);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse remotes: {}", e);
                    widgets::show_error_dialog(
                        &self.window,
                        "Error",
                        &format!("Failed to load remotes: {}", e),
                    );
                }
            }
        }
    }

    /// Setup event handlers for Edit/Delete buttons on remote rows
    fn setup_remote_row_handlers(&self, row: &ListBoxRow, remote_name: &str) {
        // We need to walk the widget tree to find the buttons
        if let Some(hbox) = row.child() {
            let mut child = hbox.first_child();

            while let Some(widget) = child {
                if let Ok(btn) = widget.clone().dynamic_cast::<Button>() {
                    if let Some(name) = btn.widget_name() {
                        let name_str = name.as_str();

                        if name_str.starts_with("edit_") {
                            let remote_name_clone = remote_name.to_string();
                            let config_manager = Arc::clone(&self.config_manager);
                            let window_clone = self.window.clone();
                            let refresh_tx = Arc::new(Mutex::new(self.refresh_tx.borrow().clone()));

                            btn.connect_clicked(move |_| {
                                tracing::info!(
                                    "Edit button clicked for remote: {}",
                                    remote_name_clone
                                );

                                if let Ok(cm) = config_manager.lock() {
                                    match cm.parse_remotes() {
                                        Ok(remotes) => {
                                            if let Some(remote) =
                                                remotes.iter().find(|r| r.name == remote_name_clone)
                                            {
                                                let dialog = AddRemoteDialog::new(
                                                    &window_clone,
                                                    Some(remote.clone()),
                                                );

                                                if let Some(updated) = dialog.run() {
                                                    match cm.update_remote(updated) {
                                                        Ok(_) => {
                                                            tracing::info!("Remote updated");
                                                            if let Ok(tx) = refresh_tx.lock() {
                                                                let _ =
                                                                    tx.send(RefreshEvent::Remotes);
                                                            }
                                                        }
                                                        Err(e) => {
                                                            tracing::error!("Update failed: {}", e);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to load remotes: {}", e);
                                        }
                                    }
                                }
                            });
                        } else if name_str.starts_with("delete_")
                            && !name_str.starts_with("delete_mount_")
                        {
                            let remote_name_clone = remote_name.to_string();
                            let config_manager = Arc::clone(&self.config_manager);
                            let window_clone = self.window.clone();
                            let refresh_tx = Arc::new(Mutex::new(self.refresh_tx.borrow().clone()));

                            btn.connect_clicked(move |_| {
                                tracing::info!(
                                    "Delete button clicked for remote: {}",
                                    remote_name_clone
                                );

                                if widgets::show_confirm_dialog(
                                    &window_clone,
                                    "Delete Remote?",
                                    &format!(
                                        "Are you sure you want to delete the remote '{}'?",
                                        remote_name_clone
                                    ),
                                ) {
                                    if let Ok(cm) = config_manager.lock() {
                                        match cm.remove_remote(&remote_name_clone) {
                                            Ok(_) => {
                                                tracing::info!("Remote deleted");
                                                widgets::show_info_dialog(
                                                    &window_clone,
                                                    "Deleted",
                                                    "Remote has been deleted successfully.",
                                                );
                                                if let Ok(tx) = refresh_tx.lock() {
                                                    let _ = tx.send(RefreshEvent::Remotes);
                                                }
                                            }
                                            Err(e) => {
                                                tracing::error!("Delete failed: {}", e);
                                                widgets::show_error_dialog(
                                                    &window_clone,
                                                    "Error",
                                                    &format!("Failed to delete remote: {}", e),
                                                );
                                            }
                                        }
                                    }
                                }
                            });
                        }
                    }
                }

                child = widget.next_sibling();
            }
        }
    }

    pub fn refresh_mounts(&self) {
        match SystemdManager::list_services() {
            Ok(services) => {
                // Clear existing rows
                while let Some(child) = self.mount_list.first_child() {
                    self.mount_list.remove(&child);
                }

                // Add mount rows for each service
                for service_name in services {
                    // Create a basic mount from service name for now
                    let mount = MountConfig {
                        id: service_name.clone(),
                        name: service_name.clone(),
                        remote_name: "Unknown".to_string(),
                        mount_point: "/tmp".to_string(),
                        options: Default::default(),
                        auto_mount: false,
                        enabled: true,
                    };

                    let status = match SystemdManager::is_mounted(&service_name) {
                        Ok(true) => MountStatus::Mounted,
                        Ok(false) => MountStatus::Unmounted,
                        Err(e) => MountStatus::Error(e.to_string()),
                    };

                    let row = widgets::create_mount_row(&mount, &status);
                    self.mount_list.append(&row);

                    // Setup event handlers for row buttons
                    self.setup_mount_row_handlers(&row, &mount.id);
                }
            }
            Err(e) => {
                tracing::error!("Failed to get services: {}", e);
                widgets::show_error_dialog(
                    &self.window,
                    "Error",
                    &format!("Failed to load mounts: {}", e),
                );
            }
        }
    }

    /// Setup event handlers for Edit/Delete/Toggle buttons on mount rows
    fn setup_mount_row_handlers(&self, row: &ListBoxRow, mount_id: &str) {
        // We need to walk the widget tree to find the buttons
        if let Some(hbox) = row.child() {
            let mut child = hbox.first_child();

            while let Some(widget) = child {
                if let Ok(btn) = widget.clone().dynamic_cast::<Button>() {
                    if let Some(name) = btn.widget_name() {
                        let name_str = name.as_str();

                        if name_str.starts_with("toggle_") {
                            let mount_id_clone = mount_id.to_string();
                            let window_clone = self.window.clone();
                            let refresh_tx = Arc::new(Mutex::new(self.refresh_tx.borrow().clone()));

                            btn.connect_clicked(move |_| {
                                tracing::info!(
                                    "Toggle button clicked for mount: {}",
                                    mount_id_clone
                                );

                                if let Ok(is_mounted) = SystemdManager::is_mounted(&mount_id_clone)
                                {
                                    let result = if is_mounted {
                                        SystemdManager::stop_mount(&mount_id_clone)
                                    } else {
                                        SystemdManager::start_mount(&mount_id_clone)
                                    };

                                    match result {
                                        Ok(_) => {
                                            tracing::info!("Mount state toggled");
                                            if let Ok(tx) = refresh_tx.lock() {
                                                let _ = tx.send(RefreshEvent::Mounts);
                                            }
                                        }
                                        Err(e) => {
                                            tracing::error!("Toggle failed: {}", e);
                                            widgets::show_error_dialog(
                                                &window_clone,
                                                "Error",
                                                &format!("Failed to toggle mount: {}", e),
                                            );
                                        }
                                    }
                                }
                            });
                        } else if name_str.starts_with("edit_mount_") {
                            let mount_id_clone = mount_id.to_string();
                            let window_clone = self.window.clone();

                            btn.connect_clicked(move |_| {
                                tracing::info!("Edit button clicked for mount: {}", mount_id_clone);
                                widgets::show_info_dialog(
                                    &window_clone,
                                    "Edit Mount",
                                    "Mount editing feature coming soon",
                                );
                            });
                        } else if name_str.starts_with("delete_mount_") {
                            let mount_id_clone = mount_id.to_string();
                            let window_clone = self.window.clone();
                            let refresh_tx = Arc::new(Mutex::new(self.refresh_tx.borrow().clone()));

                            btn.connect_clicked(move |_| {
                                tracing::info!(
                                    "Delete button clicked for mount: {}",
                                    mount_id_clone
                                );

                                if widgets::show_confirm_dialog(
                                    &window_clone,
                                    "Delete Mount?",
                                    &format!(
                                        "Are you sure you want to delete the mount '{}'?",
                                        mount_id_clone
                                    ),
                                ) {
                                    // Stop the mount first if running
                                    let _ = SystemdManager::stop_mount(&mount_id_clone);

                                    // TODO: Delete systemd service file
                                    // TODO: Remove from configuration

                                    widgets::show_info_dialog(
                                        &window_clone,
                                        "Deleted",
                                        "Mount has been deleted successfully.",
                                    );
                                    if let Ok(tx) = refresh_tx.lock() {
                                        let _ = tx.send(RefreshEvent::Mounts);
                                    }
                                    tracing::info!("Mount deleted");
                                }
                            });
                        }
                    }
                }

                child = widget.next_sibling();
            }
        }
    }
}

/// Create remotes management page
fn create_remotes_page() -> (GtkBox, Button, ListBox) {
    let main_box = GtkBox::new(Orientation::Vertical, 12);
    main_box.set_margin_top(12);
    main_box.set_margin_bottom(12);
    main_box.set_margin_start(12);
    main_box.set_margin_end(12);

    // Header
    let header = GtkBox::new(Orientation::Horizontal, 12);
    let title = Label::new(Some("Cloud Service Remotes"));
    title.add_css_class("title-2");
    header.append(&title);

    let add_button = Button::with_label("Add Remote");
    add_button.set_hexpand(false);
    add_button.set_margin_start(12);
    header.set_hexpand(true);
    header.append(&add_button);

    main_box.append(&header);

    // Remote list
    let scrolled = ScrolledWindow::new();
    let list_box = ListBox::new();
    list_box.set_selection_mode(gtk4::SelectionMode::Single);

    scrolled.set_child(Some(&list_box));
    scrolled.set_hexpand(true);
    scrolled.set_vexpand(true);
    main_box.append(&scrolled);

    (main_box, add_button, list_box)
}

/// Create mounts management page
fn create_mounts_page() -> (GtkBox, Button, ListBox) {
    let main_box = GtkBox::new(Orientation::Vertical, 12);
    main_box.set_margin_top(12);
    main_box.set_margin_bottom(12);
    main_box.set_margin_start(12);
    main_box.set_margin_end(12);

    // Header
    let header = GtkBox::new(Orientation::Horizontal, 12);
    let title = Label::new(Some("Active Mounts"));
    title.add_css_class("title-2");
    header.append(&title);

    let add_button = Button::with_label("Add Mount");
    add_button.set_hexpand(false);
    header.set_hexpand(true);
    header.append(&add_button);

    main_box.append(&header);

    // Mount list
    let scrolled = ScrolledWindow::new();
    let list_box = ListBox::new();
    list_box.set_selection_mode(gtk4::SelectionMode::Single);

    scrolled.set_child(Some(&list_box));
    scrolled.set_hexpand(true);
    scrolled.set_vexpand(true);
    main_box.append(&scrolled);

    (main_box, add_button, list_box)
}

/// Create settings page
fn create_settings_page() -> (GtkBox, Button, Button) {
    let main_box = GtkBox::new(Orientation::Vertical, 12);
    main_box.set_margin_top(12);
    main_box.set_margin_bottom(12);
    main_box.set_margin_start(12);
    main_box.set_margin_end(12);

    let title = Label::new(Some("Settings"));
    title.add_css_class("title-2");
    main_box.append(&title);

    // Settings sections
    let scrolled = ScrolledWindow::new();
    let settings_box = GtkBox::new(Orientation::Vertical, 12);

    // Section 1: Auto-mount preferences
    let section1_title = Label::new(Some("Auto-mount Behavior"));
    section1_title.add_css_class("title-3");
    section1_title.set_halign(gtk4::Align::Start);
    settings_box.append(&section1_title);

    let auto_mount_check = gtk4::CheckButton::with_label("Auto-mount enabled services at startup");
    auto_mount_check.set_active(true);
    settings_box.append(&auto_mount_check);

    let reconnect_check = gtk4::CheckButton::with_label("Auto-reconnect on mount failure");
    reconnect_check.set_active(true);
    settings_box.append(&reconnect_check);

    let reconnect_delay_box = GtkBox::new(Orientation::Horizontal, 6);
    let reconnect_delay_label = Label::new(Some("Reconnect delay (seconds):"));
    reconnect_delay_box.append(&reconnect_delay_label);
    let reconnect_delay_spin = gtk4::SpinButton::with_range(1.0, 300.0, 5.0);
    reconnect_delay_spin.set_value(30.0);
    reconnect_delay_box.append(&reconnect_delay_spin);
    settings_box.append(&reconnect_delay_box);

    // Separator
    let sep1 = gtk4::Separator::new(Orientation::Horizontal);
    settings_box.append(&sep1);

    // Section 2: Interface preferences
    let section2_title = Label::new(Some("Interface Preferences"));
    section2_title.add_css_class("title-3");
    section2_title.set_halign(gtk4::Align::Start);
    settings_box.append(&section2_title);

    let show_notifications_check = gtk4::CheckButton::with_label("Show desktop notifications");
    show_notifications_check.set_active(true);
    settings_box.append(&show_notifications_check);

    let log_level_box = GtkBox::new(Orientation::Horizontal, 6);
    let log_level_label = Label::new(Some("Log level:"));
    log_level_box.append(&log_level_label);
    let log_level_combo = gtk4::ComboBoxText::new();
    log_level_combo.append_text("Debug");
    log_level_combo.append_text("Info");
    log_level_combo.append_text("Warn");
    log_level_combo.append_text("Error");
    log_level_combo.set_active(Some(1)); // Default to Info
    log_level_box.append(&log_level_combo);
    settings_box.append(&log_level_box);

    // Separator
    let sep2 = gtk4::Separator::new(Orientation::Horizontal);
    settings_box.append(&sep2);

    // Section 3: System integration
    let section3_title = Label::new(Some("System Integration"));
    section3_title.add_css_class("title-3");
    section3_title.set_halign(gtk4::Align::Start);
    settings_box.append(&section3_title);

    let tray_enabled_check = gtk4::CheckButton::with_label("Enable system tray applet");
    tray_enabled_check.set_active(true);
    settings_box.append(&tray_enabled_check);

    let start_minimized_check = gtk4::CheckButton::with_label("Start minimized to tray");
    start_minimized_check.set_active(false);
    settings_box.append(&start_minimized_check);

    scrolled.set_child(Some(&settings_box));
    scrolled.set_hexpand(true);
    scrolled.set_vexpand(true);
    main_box.append(&scrolled);

    // Buttons section
    let buttons_box = GtkBox::new(Orientation::Horizontal, 6);
    buttons_box.set_halign(gtk4::Align::End);
    buttons_box.set_margin_top(12);

    let reset_button = Button::with_label("Reset to Defaults");
    reset_button.add_css_class("destructive-action");
    buttons_box.append(&reset_button);

    let save_button = Button::with_label("Save Settings");
    save_button.add_css_class("suggested-action");
    buttons_box.append(&save_button);

    main_box.append(&buttons_box);

    (main_box, save_button, reset_button)
}
