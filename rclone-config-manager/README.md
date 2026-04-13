# RClone Config Manager

A GTK4-based GUI application for managing rclone mount configurations and authentication.

## Features

- **Cloud Service Support**: Google Drive, OneDrive, Dropbox, Amazon S3, Backblaze B2, Box.com
- **Modern GTK4 UI**: Clean, responsive interface with libadwaita integration
- **Flexible Authentication**: OAuth browser flow and manual token input support
- **Systemd Integration**: Manage mounts as user-level systemd services
- **Remote Management**: Add, edit, and remove rclone remote configurations
- **Mount Control**: Start/stop/enable/disable mounts on demand

## Architecture

### Modules

- **`models/`**: Data structures for remotes, mounts, authentication, and mount status
- **`config/`**: RcloneConfigManager for reading/writing rclone.conf file
- **`auth/`**: OAuth configuration and token management
- **`services/`**: SystemdManager for systemd user service integration
- **`ui/`**: GTK4 main window, dialogs, and widgets
  - `mod.rs`: Main application window and pages
  - `dialogs.rs`: Configuration dialogs for adding/editing remotes and mounts
  - `widgets.rs`: List row widgets and helper functions

### Key Components

- **AppWindow**: Main application window with three pages:
  - Remotes: View and manage cloud service remotes
  - Mounts: View and manage active mounts
  - Settings: Application preferences

- **Dialogs**: 
  - AddRemoteDialog: Add/edit remote configurations
  - AddMountDialog: Add/edit mount configurations
  - OAuthDialog: OAuth authentication flow
  - ManualTokenDialog: Manual token entry

## Building

### Requirements

- Rust 1.70+
- GTK4 development libraries
- libadwaita development libraries
- systemd development libraries

### Build Steps

```bash
cd rclone-config-manager
cargo build --release
```

### Installation

```bash
cargo install --path .
```

## Development

### Project Structure

```
src/
├── main.rs              # Application entry point
├── models/mod.rs        # Data structures
├── config/mod.rs        # RcloneConfigManager
├── auth/mod.rs          # Authentication handlers
├── services/mod.rs      # Systemd integration
└── ui/
    ├── mod.rs           # Main window and pages
    ├── dialogs.rs       # Configuration dialogs
    └── widgets.rs       # UI components
```

### Dependencies

- `gtk4`: GTK4 bindings
- `libadwaita`: Modern GNOME app library
- `tokio`: Async runtime
- `serde`: Serialization
- `reqwest`: HTTP client for OAuth
- `tracing`: Logging
- `anyhow`: Error handling

## Configuration

The application reads/writes rclone configuration at `~/.config/rclone/rclone.conf` and manages mounts through systemd user services.

### Rclone Config Format

```ini
[remote-name]
type = service_type
property1 = value1
property2 = value2
```

### Systemd Services

Mounts are managed as user-level systemd services:

```
~/.config/systemd/user/rclone-mount-*.service
```

## License

MIT
