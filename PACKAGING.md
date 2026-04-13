# Debian Package Building Guide

This guide explains how to build and distribute `.deb` packages for `rclone-mount-tray` on Ubuntu and Debian platforms.

## Overview

The `rclone-mount-tray` package is a Rust-based system tray application for managing on-demand rclone mounts. The Debian packaging supports:

- **Ubuntu 24.04 LTS (Noble Numbat)**
- **Ubuntu 26.04 LTS (future)**
- **Debian 12+ (Bookworm and later)**

### Architectures

The package supports the following architectures:
- `amd64` (Intel/AMD 64-bit)
- `arm64` (ARM 64-bit)
- `armhf` (ARM 32-bit)
- `i386` (Intel/AMD 32-bit)

## Building Locally

### Prerequisites

Before building, ensure you have the required tools installed:

```bash
# Debian/Ubuntu build tools
sudo apt-get install build-essential debhelper cargo rustc

# For the build script
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Using the Build Script

The easiest way to build the package is using the provided build script:

```bash
cd rclone-mount-tray
./scripts/build-deb.sh
```

The script will:
1. Verify Rust is installed via rustup
2. Create the source tarball (`rclone-mount-tray_0.1.0.orig.tar.gz`)
3. Build the Debian package using `debuild`
4. Display the built packages

**Output:**
- `rclone-mount-tray_0.1.0-1_amd64.deb` - Binary package for amd64 architecture
- `rclone-mount-tray_0.1.0-1.dsc` - Debian source package description
- `rclone-mount-tray_0.1.0-1.tar.xz` - Debian source tarball

### Manual Build

If you prefer to build manually:

```bash
# Create the source tarball
cd ..
tar --exclude=.git --exclude=target --exclude=.cargo \
    -czf rclone-mount-tray_0.1.0.orig.tar.gz rclone-mount-tray/
cd rclone-mount-tray

# Build the package
debuild -d -us -uc
```

**Note:** The `-d` flag skips build-dependency checks (necessary for rustup-installed Rust)

## Debian Packaging Structure

### Key Files

```
debian/
├── control           # Package metadata and dependencies
├── changelog         # Version history for Debian
├── rules             # Build rules (dh-based makefile)
├── copyright         # License information
├── compat            # Debhelper compatibility version
├── postinst          # Post-installation script
├── postrm            # Post-removal script
├── install           # File placement rules
├── source/
│   └── format        # Source package format (3.0 quilt)
└── tests/
    ├── control       # Autopkgtest configuration
    └── basic-functionality  # Test script
```

### debian/rules

The build rules file handles:
- **Building**: `cargo build --release --locked`
- **Testing**: `cargo test --release --locked`
- **Installation**: Uses `debian/install` file for file placement
- **Cleanup**: Removes build artifacts and `.cargo` directory

Key environment variables:
- `CARGO_HOME`: Cache directory for Cargo
- `RUSTFLAGS`: Compiler flags (currently uses `lld` linker)
- `PATH`: Must include rustup's bin directory

### debian/install

File placement configuration:
```
target/release/rclone-mount-tray usr/bin
data/rclone-mount-tray.service usr/lib/systemd/user
data/rclone-mount-tray.desktop usr/share/applications
README.md usr/share/doc/rclone-mount-tray
QUICK_START.md usr/share/doc/rclone-mount-tray
```

### debian/postinst

Post-installation script that:
1. Reloads systemd user daemon
2. Displays setup instructions
3. Guides users through systemd service setup

### debian/postrm

Post-removal script that:
1. Disables the systemd user service
2. Reloads systemd user daemon

## Installation

### From Built Package

Install the built `.deb` package:

```bash
sudo apt install ./rclone-mount-tray_0.1.0-1_amd64.deb
```

### First-Time Setup

After installation, enable the systemd service:

```bash
# Enable the service for automatic startup
systemctl --user enable rclone-mount-tray

# Start the service
systemctl --user start rclone-mount-tray

# Check status
systemctl --user status rclone-mount-tray
```

### Verification

After installation, verify the package:

```bash
# Check if files are installed
dpkg -L rclone-mount-tray

# Verify binary is executable
which rclone-mount-tray
rclone-mount-tray --help

# Check systemd service
systemctl --user status rclone-mount-tray
```

## Cross-Compilation

To build for different architectures, use `debuild` with target architecture:

```bash
# Build for arm64
debuild -d -us -uc -a arm64

# Build for armhf
debuild -d -us -uc -a armhf

# Build for i386
debuild -d -us -uc -a i386
```

**Note:** This requires appropriate Rust targets and cross-compilation tools installed.

## Testing the Package

### Run Autopkgtests

After building, test the package:

```bash
autopkgtest ./rclone-mount-tray_0.1.0-1_amd64.deb -- lxd ubuntu:24.04
```

### Manual Installation Testing

In a clean environment:

```bash
# Extract package contents
dpkg-deb -x rclone-mount-tray_0.1.0-1_amd64.deb /tmp/test-install

# Check files
ls -la /tmp/test-install/usr/bin/
ls -la /tmp/test-install/usr/lib/systemd/user/
ls -la /tmp/test-install/usr/share/applications/

# Verify binary
/tmp/test-install/usr/bin/rclone-mount-tray --help
```

## Version Management

### Updating Version

To bump the version for a new release:

1. Update `Cargo.toml`:
```toml
[package]
version = "0.2.0"
```

2. Update `debian/changelog`:
```bash
dch -v 0.2.0-1 "New release features"
```

3. Rebuild:
```bash
./scripts/build-deb.sh
```

## Common Issues

### Issue: cargo: command not found

**Solution:** Ensure Rust is installed via rustup:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Issue: dpkg-buildpackage: error: Unmet build dependencies

**Solution:** Use the `-d` flag with debuild to skip dependency checks:
```bash
debuild -d -us -uc
```

### Issue: Build files in unexpected upstream changes

**Solution:** Recreate the source tarball:
```bash
cd ..
rm -f rclone-mount-tray_0.1.0.orig.tar.gz
tar --exclude=.git --exclude=target --exclude=.cargo \
    -czf rclone-mount-tray_0.1.0.orig.tar.gz rclone-mount-tray/
cd rclone-mount-tray
debuild -d -us -uc
```

## Distribution

### Publishing to PPA (Personal Package Archive)

To distribute via Ubuntu's PPA:

```bash
# Sign the .dsc file (requires GPG key)
debsign rclone-mount-tray_0.1.0-1.dsc

# Upload to PPA
dput ppa:your-username/your-ppa rclone-mount-tray_0.1.0-1_source.changes
```

### Creating a Release

Create a GitHub release with the following files:
- `rclone-mount-tray_0.1.0-1_amd64.deb`
- `rclone-mount-tray_0.1.0-1_arm64.deb`
- `rclone-mount-tray_0.1.0-1_armhf.deb`
- `rclone-mount-tray_0.1.0-1.dsc`
- `rclone-mount-tray_0.1.0.orig.tar.gz`

## File Locations

After installation, files are placed in:

| File | Location |
|------|----------|
| Binary | `/usr/bin/rclone-mount-tray` |
| Systemd service | `/usr/lib/systemd/user/rclone-mount-tray.service` |
| Desktop entry | `/usr/share/applications/rclone-mount-tray.desktop` |
| Documentation | `/usr/share/doc/rclone-mount-tray/` |
| Config file | `~/.config/rclone-mount-tray/mounts.toml` (created by application) |
| Logs | Via systemd journal (`journalctl --user -u rclone-mount-tray`) |

## Maintenance

### Build Dependencies

The package specifies build dependencies in `debian/control`:
```
Build-Depends: cargo (>= 1.70), rustc (>= 1.70)
```

When using rustup, the build script bypasses these checks with the `-d` flag. For system-wide Rust installation, install the dependencies:
```bash
sudo apt-get install cargo rustc
```

### Runtime Dependencies

The package specifies runtime dependencies:
- `systemd` - For systemd service integration
- `rclone (>= 1.60)` - For mount operations

Optional:
- `libappindicator3-1` - For system tray support (recommended)

## Debugging

### Enable verbose build output

```bash
VERBOSE=1 debuild -d -us -uc
```

### Check build logs

```bash
debuild -d -us -uc 2>&1 | tee build.log
```

### Inspect built package

```bash
dpkg-deb -x rclone-mount-tray_0.1.0-1_amd64.deb extracted/
dpkg-deb -W rclone-mount-tray_0.1.0-1_amd64.deb
```

## References

- [Debian New Maintainers' Guide](https://www.debian.org/doc/manuals/debmaint-faq/)
- [Packaging Rust Applications for Debian](https://wiki.debian.org/Cargo)
- [Debhelper Documentation](https://manpages.debian.org/debhelper)
- [Ubuntu Packaging Guide](https://wiki.ubuntu.com/Packaging)
- [systemd User Services](https://wiki.archlinux.org/title/Systemd/User)
