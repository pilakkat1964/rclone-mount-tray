// Dialog windows for add/edit operations

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, ComboBoxText, Dialog, Entry, Expander, Label, Orientation,
    ScrolledWindow, TextView,
};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, EntryRow};

use crate::models::{AuthCredentials, CloudService, MountConfig, RemoteConfig};

/// Dialog for adding/editing a remote configuration
pub struct AddRemoteDialog {
    dialog: Dialog,
    name_entry: Entry,
    service_combo: ComboBoxText,
    auth_method_combo: ComboBoxText,
}

impl AddRemoteDialog {
    pub fn new(parent_window: &impl IsA<gtk4::Window>, remote: Option<RemoteConfig>) -> Self {
        let dialog = Dialog::new();
        dialog.set_title(Some("Add Remote"));
        dialog.set_transient_for(Some(parent_window));
        dialog.set_modal(true);
        dialog.set_default_size(500, 400);

        // Main container
        let content_area = dialog.content_area();
        let main_box = GtkBox::new(Orientation::Vertical, 12);
        main_box.set_margin_top(12);
        main_box.set_margin_bottom(12);
        main_box.set_margin_start(12);
        main_box.set_margin_end(12);

        // Name field
        let name_label = Label::new(Some("Remote Name:"));
        name_label.set_halign(gtk4::Align::Start);
        main_box.append(&name_label);

        let name_entry = Entry::new();
        if let Some(ref r) = remote {
            name_entry.set_text(&r.name);
        }
        main_box.append(&name_entry);

        // Service selection
        let service_label = Label::new(Some("Cloud Service:"));
        service_label.set_halign(gtk4::Align::Start);
        main_box.append(&service_label);

        let service_combo = ComboBoxText::new();
        service_combo.append_text("Google Drive");
        service_combo.append_text("OneDrive");
        service_combo.append_text("Dropbox");
        service_combo.append_text("Amazon S3");
        service_combo.append_text("Backblaze B2");
        service_combo.append_text("Box.com");

        if let Some(ref r) = remote {
            match r.service {
                CloudService::GoogleDrive => service_combo.set_active_id(Some("0")),
                CloudService::OneDrive => service_combo.set_active_id(Some("1")),
                CloudService::Dropbox => service_combo.set_active_id(Some("2")),
                CloudService::AmazonS3 => service_combo.set_active_id(Some("3")),
                CloudService::BackBlaze => service_combo.set_active_id(Some("4")),
                CloudService::Box => service_combo.set_active_id(Some("5")),
            }
        }
        main_box.append(&service_combo);

        // Auth method selection
        let auth_label = Label::new(Some("Authentication Method:"));
        auth_label.set_halign(gtk4::Align::Start);
        main_box.append(&auth_label);

        let auth_method_combo = ComboBoxText::new();
        auth_method_combo.append_text("OAuth Browser");
        auth_method_combo.append_text("Manual Token");
        auth_method_combo.set_active(Some(0));
        main_box.append(&auth_method_combo);

        // Advanced options (collapsed by default)
        let advanced = Expander::new(Some("Advanced Options"));
        let advanced_box = GtkBox::new(Orientation::Vertical, 8);
        advanced_box.set_margin_top(8);
        advanced_box.set_margin_start(12);

        let path_label = Label::new(Some("Remote Path:"));
        path_label.set_halign(gtk4::Align::Start);
        advanced_box.append(&path_label);

        let path_entry = Entry::new();
        if let Some(ref r) = remote {
            if let Some(ref path) = r.remote_path {
                path_entry.set_text(path);
            }
        }
        advanced_box.append(&path_entry);

        advanced.set_child(Some(&advanced_box));
        main_box.append(&advanced);

        content_area.append(&main_box);

        // Dialog buttons
        dialog.add_button("Cancel", gtk4::ResponseType::Cancel as i32);
        dialog.add_button("Save", gtk4::ResponseType::Accept as i32);
        dialog.set_default_response(gtk4::ResponseType::Accept as i32);

        Self {
            dialog,
            name_entry,
            service_combo,
            auth_method_combo,
        }
    }

    pub fn run(&self) -> Option<RemoteConfig> {
        let response = self.dialog.run();
        if response == gtk4::ResponseType::Accept as i32 {
            let name = self.name_entry.text().to_string();
            let service_idx = self.service_combo.active()? as usize;
            let service = match service_idx {
                0 => CloudService::GoogleDrive,
                1 => CloudService::OneDrive,
                2 => CloudService::Dropbox,
                3 => CloudService::AmazonS3,
                4 => CloudService::BackBlaze,
                5 => CloudService::Box,
                _ => CloudService::GoogleDrive,
            };

            let oauth_method = match self.auth_method_combo.active().map(|v| v as usize) {
                Some(0) => "oauth",
                _ => "manual",
            };

            Some(RemoteConfig {
                name,
                service,
                remote_path: None,
                auth_method: oauth_method.to_string(),
                credentials: AuthCredentials::default(),
                properties: Default::default(),
            })
        } else {
            None
        }
    }
}

/// Dialog for adding/editing a mount configuration
pub struct AddMountDialog {
    dialog: Dialog,
    name_entry: Entry,
    remote_combo: ComboBoxText,
    mount_point_entry: Entry,
}

impl AddMountDialog {
    pub fn new(
        parent_window: &impl IsA<gtk4::Window>,
        remotes: &[RemoteConfig],
        mount: Option<MountConfig>,
    ) -> Self {
        let dialog = Dialog::new();
        dialog.set_title(Some("Add Mount"));
        dialog.set_transient_for(Some(parent_window));
        dialog.set_modal(true);
        dialog.set_default_size(500, 300);

        let content_area = dialog.content_area();
        let main_box = GtkBox::new(Orientation::Vertical, 12);
        main_box.set_margin_top(12);
        main_box.set_margin_bottom(12);
        main_box.set_margin_start(12);
        main_box.set_margin_end(12);

        // Mount name
        let name_label = Label::new(Some("Mount Name:"));
        name_label.set_halign(gtk4::Align::Start);
        main_box.append(&name_label);

        let name_entry = Entry::new();
        if let Some(ref m) = mount {
            name_entry.set_text(&m.name);
        }
        main_box.append(&name_entry);

        // Remote selection
        let remote_label = Label::new(Some("Remote:"));
        remote_label.set_halign(gtk4::Align::Start);
        main_box.append(&remote_label);

        let remote_combo = ComboBoxText::new();
        for (idx, remote) in remotes.iter().enumerate() {
            remote_combo.append_text(&remote.name);
            if let Some(ref m) = mount {
                if m.remote_name == remote.name {
                    remote_combo.set_active(Some(idx as u32));
                }
            }
        }
        main_box.append(&remote_combo);

        // Mount point path
        let mount_point_label = Label::new(Some("Mount Point:"));
        mount_point_label.set_halign(gtk4::Align::Start);
        main_box.append(&mount_point_label);

        let mount_point_entry = Entry::new();
        if let Some(ref m) = mount {
            mount_point_entry.set_text(&m.mount_point);
        }
        main_box.append(&mount_point_entry);

        content_area.append(&main_box);

        // Dialog buttons
        dialog.add_button("Cancel", gtk4::ResponseType::Cancel as i32);
        dialog.add_button("Save", gtk4::ResponseType::Accept as i32);
        dialog.set_default_response(gtk4::ResponseType::Accept as i32);

        Self {
            dialog,
            name_entry,
            remote_combo,
            mount_point_entry,
        }
    }

    pub fn run(&self) -> Option<MountConfig> {
        let response = self.dialog.run();
        if response == gtk4::ResponseType::Accept as i32 {
            Some(MountConfig {
                id: uuid::Uuid::new_v4().to_string(),
                name: self.name_entry.text().to_string(),
                remote_name: self
                    .remote_combo
                    .active_text()
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                mount_point: self.mount_point_entry.text().to_string(),
                options: Default::default(),
                auto_mount: false,
                enabled: true,
            })
        } else {
            None
        }
    }
}

/// Dialog for OAuth authentication
pub struct OAuthDialog {
    dialog: Dialog,
    status_label: Label,
}

impl OAuthDialog {
    pub fn new(parent_window: &impl IsA<gtk4::Window>, service: CloudService) -> Self {
        let dialog = Dialog::new();
        dialog.set_title(Some(&format!("{} Authentication", service.display_name())));
        dialog.set_transient_for(Some(parent_window));
        dialog.set_modal(true);
        dialog.set_default_size(500, 300);

        let content_area = dialog.content_area();
        let main_box = GtkBox::new(Orientation::Vertical, 12);
        main_box.set_margin_top(12);
        main_box.set_margin_bottom(12);
        main_box.set_margin_start(12);
        main_box.set_margin_end(12);

        let info = Label::new(Some(
            "A browser window will open to complete authentication.\nPlease follow the prompts.",
        ));
        info.set_wrap(true);
        main_box.append(&info);

        let status_label = Label::new(Some("Click 'Start Auth' to begin..."));
        status_label.set_wrap(true);
        main_box.append(&status_label);

        content_area.append(&main_box);

        dialog.add_button("Cancel", gtk4::ResponseType::Cancel as i32);
        dialog.add_button("Start Auth", gtk4::ResponseType::Apply as i32);
        dialog.set_default_response(gtk4::ResponseType::Apply as i32);

        Self {
            dialog,
            status_label,
        }
    }

    pub fn update_status(&self, message: &str) {
        self.status_label.set_label(message);
    }

    pub fn run(&self) -> gtk4::ResponseType {
        let response = self.dialog.run();
        gtk4::ResponseType::from(response as u32)
    }
}

/// Dialog for manual token entry
pub struct ManualTokenDialog {
    dialog: Dialog,
    token_entry: Entry,
}

impl ManualTokenDialog {
    pub fn new(parent_window: &impl IsA<gtk4::Window>, service: CloudService) -> Self {
        let dialog = Dialog::new();
        dialog.set_title(Some(&format!("{} Manual Token", service.display_name())));
        dialog.set_transient_for(Some(parent_window));
        dialog.set_modal(true);
        dialog.set_default_size(500, 200);

        let content_area = dialog.content_area();
        let main_box = GtkBox::new(Orientation::Vertical, 12);
        main_box.set_margin_top(12);
        main_box.set_margin_bottom(12);
        main_box.set_margin_start(12);
        main_box.set_margin_end(12);

        let label = Label::new(Some("Paste your authentication token:"));
        label.set_halign(gtk4::Align::Start);
        main_box.append(&label);

        let token_entry = Entry::new();
        token_entry.set_visibility(false);
        token_entry.set_placeholder_text(Some("Enter token"));
        main_box.append(&token_entry);

        content_area.append(&main_box);

        dialog.add_button("Cancel", gtk4::ResponseType::Cancel as i32);
        dialog.add_button("Save Token", gtk4::ResponseType::Accept as i32);
        dialog.set_default_response(gtk4::ResponseType::Accept as i32);

        Self {
            dialog,
            token_entry,
        }
    }

    pub fn run(&self) -> Option<String> {
        let response = self.dialog.run();
        if response == gtk4::ResponseType::Accept as i32 {
            Some(self.token_entry.text().to_string())
        } else {
            None
        }
    }
}
