// Helper widgets and builders

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, IconTheme, Image, Label, ListBox, ListBoxRow, Orientation, ResponseType,
    Spinner,
};

use crate::models::{MountConfig, MountStatus, RemoteConfig};

/// Create a remote list row with embedded remote name for data retrieval
pub fn create_remote_row(remote: &RemoteConfig) -> ListBoxRow {
    let row = ListBoxRow::new();
    let hbox = GtkBox::new(Orientation::Horizontal, 12);
    hbox.set_margin_top(6);
    hbox.set_margin_bottom(6);
    hbox.set_margin_start(6);
    hbox.set_margin_end(6);

    // Service icon and name
    let icon_label = Label::new(Some(&remote.service.icon_char()));
    icon_label.add_css_class("title-4");
    hbox.append(&icon_label);

    let vbox = GtkBox::new(Orientation::Vertical, 2);
    let name_label = Label::new(Some(&remote.name));
    name_label.set_halign(gtk4::Align::Start);
    name_label.add_css_class("title-4");
    vbox.append(&name_label);

    let service_label = Label::new(Some(remote.service.display_name()));
    service_label.set_halign(gtk4::Align::Start);
    service_label.add_css_class("dim-label");
    vbox.append(&service_label);

    hbox.append(&vbox);
    hbox.set_hexpand(true);

    // Action buttons
    let edit_btn = Button::with_label("Edit");
    edit_btn.add_css_class("suggested-action");
    edit_btn.set_name(&format!("edit_{}", remote.name));
    hbox.append(&edit_btn);

    let delete_btn = Button::with_label("Delete");
    delete_btn.add_css_class("destructive-action");
    delete_btn.set_name(&format!("delete_{}", remote.name));
    hbox.append(&delete_btn);

    row.set_child(Some(&hbox));

    // Store remote name on row for later retrieval
    row.set_name(&format!("remote_{}", remote.name));

    row
}

/// Create a mount list row with embedded mount ID for data retrieval
pub fn create_mount_row(mount: &MountConfig, status: &MountStatus) -> ListBoxRow {
    let row = ListBoxRow::new();
    let hbox = GtkBox::new(Orientation::Horizontal, 12);
    hbox.set_margin_top(6);
    hbox.set_margin_bottom(6);
    hbox.set_margin_start(6);
    hbox.set_margin_end(6);

    // Status indicator
    let status_indicator = create_status_indicator(status);
    hbox.append(&status_indicator);

    // Mount info
    let vbox = GtkBox::new(Orientation::Vertical, 2);
    let name_label = Label::new(Some(&mount.name));
    name_label.set_halign(gtk4::Align::Start);
    name_label.add_css_class("title-4");
    vbox.append(&name_label);

    let path_label = Label::new(Some(&mount.mount_point));
    path_label.set_halign(gtk4::Align::Start);
    path_label.add_css_class("monospace");
    path_label.add_css_class("dim-label");
    vbox.append(&path_label);

    let remote_label = Label::new(Some(&mount.remote_name));
    remote_label.set_halign(gtk4::Align::Start);
    remote_label.add_css_class("dim-label");
    vbox.append(&remote_label);

    hbox.append(&vbox);
    hbox.set_hexpand(true);

    // Action buttons
    let action_box = GtkBox::new(Orientation::Horizontal, 6);
    let toggle_btn = match status {
        MountStatus::Mounted => Button::with_label("Unmount"),
        MountStatus::Unmounted => Button::with_label("Mount"),
        MountStatus::Mounting | MountStatus::Unmounting => Button::with_label("Mounting..."),
        MountStatus::Error(_) => Button::with_label("Retry"),
    };
    toggle_btn.set_name(&format!("toggle_{}", mount.id));
    action_box.append(&toggle_btn);

    let edit_btn = Button::with_label("Edit");
    edit_btn.add_css_class("suggested-action");
    edit_btn.set_name(&format!("edit_mount_{}", mount.id));
    action_box.append(&edit_btn);

    let delete_btn = Button::with_label("Delete");
    delete_btn.add_css_class("destructive-action");
    delete_btn.set_name(&format!("delete_mount_{}", mount.id));
    action_box.append(&delete_btn);

    hbox.append(&action_box);

    row.set_child(Some(&hbox));

    // Store mount ID on row for later retrieval
    row.set_name(&format!("mount_{}", mount.id));

    row
}

/// Create a status indicator widget
fn create_status_indicator(status: &MountStatus) -> GtkBox {
    let indicator = GtkBox::new(Orientation::Vertical, 0);
    indicator.set_size_request(24, 24);
    indicator.add_css_class("status-indicator");

    match status {
        MountStatus::Mounted => {
            indicator.add_css_class("success");
            let label = Label::new(Some("✓"));
            label.add_css_class("title-2");
            indicator.append(&label);
        }
        MountStatus::Unmounted => {
            indicator.add_css_class("dim");
            let label = Label::new(Some("—"));
            label.add_css_class("title-2");
            indicator.append(&label);
        }
        MountStatus::Mounting | MountStatus::Unmounting => {
            indicator.add_css_class("dim");
            let spinner = Spinner::new();
            spinner.set_spinning(true);
            indicator.append(&spinner);
        }
        MountStatus::Error(_) => {
            indicator.add_css_class("error");
            let label = Label::new(Some("!"));
            label.add_css_class("title-2");
            indicator.append(&label);
        }
    }

    indicator
}

/// Create a loading spinner
pub fn create_spinner() -> Spinner {
    let spinner = Spinner::new();
    spinner.set_spinning(true);
    spinner
}

/// Message dialog helper
pub fn show_error_dialog(parent: &impl IsA<gtk4::Window>, title: &str, message: &str) {
    let dialog = gtk4::MessageDialog::new(
        Some(parent),
        gtk4::DialogFlags::MODAL,
        gtk4::MessageType::Error,
        gtk4::ButtonsType::Ok,
        title,
    );
    dialog.set_secondary_text(Some(message));
    dialog.run_async(|dialog, _| {
        dialog.close();
    });
}

/// Info dialog helper
pub fn show_info_dialog(parent: &impl IsA<gtk4::Window>, title: &str, message: &str) {
    let dialog = gtk4::MessageDialog::new(
        Some(parent),
        gtk4::DialogFlags::MODAL,
        gtk4::MessageType::Info,
        gtk4::ButtonsType::Ok,
        title,
    );
    dialog.set_secondary_text(Some(message));
    dialog.run_async(|dialog, _| {
        dialog.close();
    });
}

/// Confirmation dialog helper
pub fn show_confirm_dialog(parent: &impl IsA<gtk4::Window>, title: &str, message: &str) -> bool {
    let dialog = gtk4::MessageDialog::new(
        Some(parent),
        gtk4::DialogFlags::MODAL,
        gtk4::MessageType::Question,
        gtk4::ButtonsType::YesNo,
        title,
    );
    dialog.set_secondary_text(Some(message));
    let response = dialog.run();
    dialog.close();
    response == ResponseType::Yes as i32
}
