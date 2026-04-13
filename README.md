# RClone Mount Manager - System Tray & GUI

A complete rclone mount management solution with both a lightweight system tray applet and a modern GTK4-based GUI application, both written in Rust. Seamlessly integrates with your existing rclone on-demand mounting system.

## Components

### 1. **System Tray Applet** (`rclone-mount-tray`)
A lightweight system tray application for quick status monitoring and mount control.

### 2. **Configuration Manager** (`rclone-config-manager`)
A modern GTK4 GUI for managing rclone remotes, mounts, and authentication.

## Features

✨ **Core Features:**
- **System Tray Integration** - Display rclone mount status in the system tray
- **Mount Status Indicator** - Visual feedback showing overall mount status
  - 🟢 Green: All mounts active
  - 🟡 Orange: Partial mounts active
  - 🔴 Red: Error or unmounted
- **One-Click Toggle** - Mount/unmount drives directly from the tray menu
- **Auto-Refresh** - Status updates every 5 seconds
- **Minimal Resource Usage** - Written in Rust with minimal dependencies

## System Requirements

- **Linux** (X11 or Wayland)
- **Rust 1.94.1 or later** (for building)
- **systemd user services** enabled
- **rclone** installed and configured

### Tested Desktop Environments
- KDE Plasma 5.x / 6.x
- GNOME 40+
- XFCE (with system tray)

### Optional Dependencies
- `libappindicator3` (for better system tray integration on some DEs)

## Installation

### From Source

1. **Clone and build:**
   ```bash
   cd ~/workspace/rclone-mount-tray
   cargo build --release
   ```

2. **Install the binary:**
   ```bash
   mkdir -p ~/.local/bin
   cp target/release/rclone-mount-tray ~/.local/bin/
   chmod +x ~/.local/bin/rclone-mount-tray
   ```

3. **Install desktop entry (for application menu):**
   ```bash
   mkdir -p ~/.local/share/applications
   cp data/rclone-mount-tray.desktop ~/.local/share/applications/
   ```

4. **Install systemd service (for auto-start):**
   ```bash
   mkdir -p ~/.config/systemd/user
   cp data/rclone-mount-tray.service ~/.config/systemd/user/
   systemctl --user daemon-reload
   systemctl --user enable --now rclone-mount-tray.service
   ```

## Usage

### Manual Start

```bash
~/.local/bin/rclone-mount-tray
```

Or from your application menu: **RClone Mount Manager**

### Auto-Start with Session

The applet is configured to auto-start via systemd user service. To enable/disable:

```bash
# Enable auto-start
systemctl --user enable rclone-mount-tray.service

# Disable auto-start
systemctl --user disable rclone-mount-tray.service

# Check status
systemctl --user status rclone-mount-tray.service

# View logs
journalctl --user -u rclone-mount-tray.service -f
```

### System Tray Usage

1. **View Status** - The tray icon shows the overall mount state
2. **Click Icon** - Opens the context menu
3. **Mount/Unmount** - Click a drive name to toggle its status
4. **Refresh** - Manually refresh the status
5. **Quit** - Exit the application

## Configuration Manager GUI

The `rclone-config-manager` provides a full-featured GTK4 interface for:

- **Remote Management**: Add, edit, and remove rclone remote configurations
- **Cloud Services**: Support for Google Drive, OneDrive, Dropbox, S3, B2, Box.com
- **Authentication**: 
  - OAuth browser-based flow with automatic browser launch
  - Manual token input for custom authentication
- **Mount Control**: Create, manage, and control rclone mounts
- **Systemd Integration**: Manage mounts as user-level systemd services

### Using the GUI

1. **Launch**: Application menu → RClone Config Manager
2. **Configure Remotes**: Remotes tab → Add Remote
3. **Create Mounts**: Mounts tab → Add Mount
4. **Control Mounts**: Mount/Unmount/Edit/Delete operations
5. **Authentication**: OAuth flow with browser integration or manual token entry

## Configuration

### Mount Configuration File

The applet reads mount configuration from:
```
~/.config/rclone-mount-tray/mounts.toml
```

If the file doesn't exist, the applet uses default mounts from the bash script configuration.

**Example mounts.toml:**
```toml
[[mounts]]
remote = "gdrive_pilakkat"
mount_point = "/home/user/gdrive_pilakkat"

[[mounts]]
remote = "gdrive_goofybits"
mount_point = "/home/user/gdrive_goofybits"
```

### Adding New Mounts

1. Edit `~/.config/rclone-mount-tray/mounts.toml`
2. Add new entries following the format above
3. Restart the applet or wait for auto-refresh

### Logging

Logs are written to systemd journal:

```bash
# View recent logs
journalctl --user -u rclone-mount-tray.service -n 50

# Follow logs in real-time
journalctl --user -u rclone-mount-tray.service -f

# View debug logs
journalctl --user -u rclone-mount-tray.service --all
```

## Architecture

### Design

```
┌─────────────────────────────────┐
│  RClone Mount Manager Tray App  │
│         (Rust / Tokio)          │
├─────────────────────────────────┤
│                                 │
│  ┌─ Tray UI Layer               │
│  │  ├─ System tray icon         │
│  │  ├─ Context menu             │
│  │  └─ Status display           │
│  │                              │
│  ├─ Mount Manager               │
│  │  ├─ Config loading/parsing   │
│  │  ├─ Mount status checking    │
│  │  └─ Mount size calculation   │
│  │                              │
│  └─ Systemd Integration         │
│     ├─ Service start/stop       │
│     └─ Service status queries   │
│                                 │
└─────────────────────────────────┘
         │
         ├─ Communicates with systemd user services
         ├─ Reads /proc/mounts for status
         └─ Executes rclone commands
```

### Modules

- **mount_manager.rs** - Rclone mount configuration and status checking
- **systemd_manager.rs** - Systemd user service communication
- **tray_ui.rs** - System tray UI and menu management
- **main.rs** - Application entry point and main event loop

## Troubleshooting

### Applet doesn't appear in system tray

1. **Check if running:**
   ```bash
   pgrep -f rclone-mount-tray
   ```

2. **Check logs:**
   ```bash
   journalctl --user -u rclone-mount-tray.service -n 20
   ```

3. **Try running manually:**
   ```bash
   ~/.local/bin/rclone-mount-tray
   ```

### Status doesn't update

1. **Verify systemd services exist:**
   ```bash
   systemctl --user list-units | grep rclone
   ```

2. **Check mount status manually:**
   ```bash
   source ~/.local/bin/rclone-mount-manager.sh
   rclone-status
   ```

### Mount/Unmount doesn't work

1. **Verify systemd service permissions:**
   ```bash
   systemctl --user status rclone-gdrive-gdrive_pilakkat.service
   ```

2. **Check if service is accessible:**
   ```bash
   systemctl --user start rclone-gdrive-gdrive_pilakkat.service
   ```

3. **Review applet logs:**
   ```bash
   journalctl --user -u rclone-mount-tray.service -f
   ```

### High CPU usage

The applet runs a refresh loop every 5 seconds. If you notice high CPU:

1. Check if a command is hanging:
   ```bash
   ps aux | grep rclone-mount-tray
   ```

2. Look for systemd service issues:
   ```bash
   journalctl --user -u rclone-mount-tray.service --all
   ```

## Integration with Bash Mount Manager

This applet works seamlessly with the bash mount manager script (`~/.local/bin/rclone-mount-manager.sh`).

Both tools can be used interchangeably:

```bash
# Use the bash script to mount
rclone-mount gdrive_pilakkat

# The tray applet will reflect the change
# (status updates within 5 seconds)

# Or use the applet to unmount via GUI
# The bash script will show the updated status
rclone-status
```

## Building from Source

### Requirements

- Rust 1.94.1+
- Linux development headers
- GTK 3+ or 4 (optional, for enhanced menu support)

### Build Commands

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Build documentation
cargo doc --open
```

## Performance

- **Memory Usage** - ~20-30 MB at rest
- **CPU Usage** - <1% in idle state
- **Startup Time** - <500ms
- **Status Refresh Interval** - 5 seconds

## Security Notes

- The applet communicates with local systemd services only
- No network access except to rclone mounts
- rclone OAuth tokens are stored in `~/.config/rclone/rclone.conf`
- Credentials are managed by rclone, not this applet

## Known Limitations

- **GNOME 45+** - Limited system tray support (consider using extension)
- **Wayland** - Tray display varies by compositor
- **Flatpak** - Would require portals for systemd access

## Future Enhancements

- [ ] Settings UI for managing mounts
- [ ] Custom keyboard shortcuts
- [ ] Mount size tracking and display
- [ ] Sound notifications on mount/unmount
- [ ] Systemd timer integration for scheduled mounting
- [ ] Custom icon support
- [ ] Multi-language support

## License

MIT License - See LICENSE file for details

## Contributing

Contributions welcome! Please feel free to submit issues or pull requests.

## Related Projects

- **rclone** - The main rclone project
- **rclone-mount-manager.sh** - Bash script for on-demand mounting
- **RCLONE_ONDEMAND_GUIDE.md** - User guide for on-demand mounting

## CI/CD and Releases

### Continuous Integration

This project uses GitHub Actions for automated building, testing, and releasing.

**Workflows:**
- **CI Build** - Tests on every push/PR (tests, linting, security audit)
- **Debian Packages** - Validates Debian packaging
- **Release** - Automated builds on version tags

See [CI_CD.md](CI_CD.md) for complete details.

### Getting Releases

**From GitHub Releases:**
```bash
# Download latest release
gh release download -p "*.deb"

# Install
sudo apt install ./rclone-mount-tray_*.deb
```

**Build Status:** [![CI](https://github.com/pilakkat1964/rclone-mount-tray/actions/workflows/ci.yml/badge.svg)](https://github.com/pilakkat1964/rclone-mount-tray/actions)

### Creating a Release

See [RELEASE_PROCEDURE.md](RELEASE_PROCEDURE.md) for detailed steps.

Quick process:
```bash
# Update version
vim Cargo.toml debian/changelog

# Commit and tag
git add Cargo.toml debian/changelog
git commit -m "chore: Release v0.2.0"
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin master v0.2.0
```

## Documentation

- **[README.md](README.md)** - This file
- **[QUICK_START.md](QUICK_START.md)** - 5-minute getting started guide
- **[PACKAGING.md](PACKAGING.md)** - Debian packaging guide
- **[CI_CD.md](CI_CD.md)** - CI/CD pipeline documentation
- **[RELEASE_PROCEDURE.md](RELEASE_PROCEDURE.md)** - Release procedures
- **[RCLONE_ONDEMAND_GUIDE.md](../RCLONE_ONDEMAND_GUIDE.md)** - On-demand mounting guide

## Support

For issues, suggestions, or questions:

1. Check existing documentation
2. Review logs via `journalctl`
3. Test with bash mount manager to isolate issues
4. [Create an issue](https://github.com/pilakkat1964/rclone-mount-tray/issues) on GitHub
5. Report issues with relevant logs and environment info

---

**Happy mounting!** 📦
