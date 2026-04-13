# GTK4 System Dependencies Installation Guide

## Problem

The GUI application `rclone-config-manager` requires GTK4 system libraries to compile and run. Without these libraries installed, compilation fails with errors like:

```
The system library `gtk4` required by crate `gdk4-sys` was not found.
The file `gtk4.pc' needs to be installed
```

## Required System Packages

### Debian/Ubuntu

```bash
# Install GTK4 development files and dependencies
sudo apt-get update
sudo apt-get install -y \
    libgtk-4-dev \
    libglib2.0-dev \
    libgraphene-1-dev \
    libadwaita-1-dev \
    libpango1.0-dev \
    libcairo2-dev \
    pkg-config
```

### Fedora/RHEL

```bash
sudo dnf install -y \
    gtk4-devel \
    glib2-devel \
    graphene-devel \
    libadwaita-devel \
    pango-devel \
    cairo-devel \
    pkg-config
```

### Arch Linux

```bash
sudo pacman -S \
    gtk4 \
    glib2 \
    graphene \
    libadwaita \
    pango \
    cairo \
    pkg-config
```

### macOS (with Homebrew)

```bash
brew install gtk4 libadwaita pkg-config
```

## Troubleshooting

### Error: pkg-config exited with status code 1

If you get this error after installing packages, set the PKG_CONFIG_PATH:

```bash
# For Debian/Ubuntu
export PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig:$PKG_CONFIG_PATH"

# Then try building again
cd rclone-config-manager
cargo build
```

### Error: file graphene-gobject-1.0.pc' was not found

Install graphene development files:

```bash
# Debian/Ubuntu
sudo apt-get install libgraphene-1-dev

# Or verify installation
pkg-config --list-all | grep graphene
```

### Error: file libadwaita-1.pc' was not found

Install libadwaita development files:

```bash
# Debian/Ubuntu
sudo apt-get install libadwaita-1-dev

# Verify
pkg-config --list-all | grep adwaita
```

## Verification

After installation, verify GTK4 is available:

```bash
# Check if pkg-config can find GTK4
pkg-config --modversion gtk4

# Expected output: 4.x.x (version number)
```

## Building with GTK4 Installed

Once all system libraries are installed:

```bash
cd rclone-config-manager

# Check if it compiles
cargo check

# Build in release mode
cargo build --release

# Run the application
./target/release/rclone-config-manager
```

## Alternative: Using Docker

If installing system libraries is not possible, use Docker to build and run:

```dockerfile
FROM rust:latest

RUN apt-get update && apt-get install -y \
    libgtk-4-dev \
    libadwaita-1-dev \
    pkg-config

WORKDIR /app
COPY . .

RUN cd rclone-config-manager && cargo build --release

CMD ["./target/release/rclone-config-manager"]
```

Build and run with Docker:

```bash
docker build -t rclone-config-manager .
docker run -e DISPLAY=$DISPLAY \
    -v /tmp/.X11-unix:/tmp/.X11-unix \
    rclone-config-manager
```

## Continuous Integration (GitHub Actions)

For CI/CD, include GTK4 installation in your workflow:

```yaml
- name: Install GTK4 dependencies
  run: |
    sudo apt-get update
    sudo apt-get install -y \
      libgtk-4-dev \
      libadwaita-1-dev \
      pkg-config

- name: Build GUI application
  run: |
    cd rclone-config-manager
    cargo build --release
```

## Feature Flags (Optional)

To make GTK4 optional for headless servers, you can use Cargo features:

```toml
[features]
default = ["gui"]
gui = ["gtk4", "libadwaita", "gio"]
cli = []
```

Then in main.rs:

```rust
#[cfg(feature = "gui")]
mod ui;

#[cfg(feature = "gui")]
fn main() {
    ui::run_gui();
}

#[cfg(not(feature = "gui"))]
fn main() {
    eprintln!("GUI feature not enabled. Build with: cargo build --features gui");
}
```

Build headless:

```bash
cargo build --no-default-features --features cli
```

Build with GUI:

```bash
cargo build --features gui
```

## Common Issues

### 1. Multiple GTK Versions

If you have GTK3 and GTK4 installed:

```bash
# Verify which version pkg-config finds
pkg-config --modversion gtk4

# If it still finds GTK3, explicitly set path
export PKG_CONFIG_PATH="/usr/lib/x86_64-linux-gnu/pkgconfig"
pkg-config --modversion gtk4
```

### 2. Development Machine vs. System Requirements

The system running the GUI needs GTK4 runtime libraries (not just dev files):

- **Build Machine**: Needs libgtk-4-dev (development headers)
- **Runtime Machine**: Needs libgtk-4-1 (runtime library)

If distributing as .deb package, add runtime dependency:

```
Depends: libgtk-4-1, libadwaita-1
```

### 3. Wayland vs. X11

GTK4 works with both Wayland and X11. If using Docker/remote display:

```bash
# For X11 forwarding
docker run --net=host \
    -e DISPLAY=$DISPLAY \
    -v /tmp/.X11-unix:/tmp/.X11-unix:rw \
    rclone-config-manager

# For Wayland
docker run -e WAYLAND_DISPLAY=$WAYLAND_DISPLAY \
    -v $XDG_RUNTIME_DIR:$XDG_RUNTIME_DIR:rw \
    rclone-config-manager
```

## Testing Without Full GTK4 Installation

If you want to test the non-GUI components without GTK4:

```bash
# Run tests for config and models modules (skip GUI)
cargo test --lib config
cargo test --lib models

# This doesn't require GTK4 libraries
```

## Performance Considerations

GTK4 libraries add ~150MB of dependencies. For minimal deployment:

- Use `--release` builds (smaller binary)
- Strip symbols: `strip target/release/rclone-config-manager`
- Consider AppImage or Flatpak for distribution (handles dependencies)

## Distribution via AppImage

Create standalone executable:

```bash
# Install linuxdeploy
curl -L -o linuxdeploy-x86_64.AppImage \
    https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
chmod +x linuxdeploy-x86_64.AppImage

# Create AppImage
./linuxdeploy-x86_64.AppImage \
    --appdir=AppDir \
    --executable=target/release/rclone-config-manager \
    --icon-file=data/icon.svg \
    --desktop-file=data/rclone-config-manager.desktop \
    --output=appimage
```

This creates a single `.AppImage` file with all dependencies bundled.

## References

- [GTK4 Official Documentation](https://docs.gtk.org/gtk4/)
- [Libadwaita Documentation](https://gnome.pages.gitlab.gnome.org/libadwaita/)
- [gtk-rs Getting Started](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/)
- [pkg-config Manual](https://linux.die.net/man/1/pkg-config)
