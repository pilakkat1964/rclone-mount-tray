#!/bin/bash
# Build script for rclone-mount-tray Debian packages
# This script handles building .deb packages in an environment where Rust
# may be installed via rustup rather than system packages

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
PACKAGE_NAME="rclone-mount-tray"
VERSION=$(grep '^version' "$PROJECT_DIR/Cargo.toml" | head -1 | sed 's/.*"\([^"]*\)".*/\1/')
DEBIAN_VERSION="${VERSION}-1"

echo "Building $PACKAGE_NAME version $VERSION..."
echo "Debian package version: $DEBIAN_VERSION"

# Ensure we have Rust available
if ! command -v cargo &> /dev/null; then
    echo "ERROR: cargo not found. Please install Rust via rustup:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Ensure Cargo.lock exists
if [ ! -f "$PROJECT_DIR/Cargo.lock" ]; then
    echo "ERROR: Cargo.lock not found. Run 'cargo generate-lockfile' first"
    exit 1
fi

# Create orig tarball if it doesn't exist
ORIG_TARBALL="../${PACKAGE_NAME}_${VERSION}.orig.tar.gz"
if [ ! -f "$PROJECT_DIR/$ORIG_TARBALL" ]; then
    echo "Creating source tarball..."
    cd "$PROJECT_DIR/.."
    tar --exclude=.git --exclude=target --exclude=.cargo -czf \
        "${PACKAGE_NAME}_${VERSION}.orig.tar.gz" \
        "$PACKAGE_NAME/"
    cd "$PROJECT_DIR"
fi

# Build the Debian package
echo "Building Debian package..."
cd "$PROJECT_DIR"

# Use dbuild with -d flag to skip build-dependency checks  
# This is necessary when Rust is installed via rustup
debuild -d -us -uc

echo "Build complete!"
echo "Packages created in $(dirname "$PROJECT_DIR"):"
ls -lh ../rclone-mount-tray*.deb 2>/dev/null || echo "No .deb files found"
