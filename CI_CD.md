# CI/CD Pipeline Documentation

This document describes the GitHub Actions CI/CD pipeline for automated building, testing, and releasing of rclone-mount-tray.

## Overview

The CI/CD pipeline consists of three main workflows:

1. **CI Build and Test** (`ci.yml`) - Runs on every push and PR
2. **Release Build** (`release.yml`) - Runs on version tags (v*.*.*)
3. **Debian Package Tests** (`debian-tests.yml`) - Validates Debian packaging

### Repository

GitHub: https://github.com/pilakkat1964/rclone-mount-tray

## Workflows

### 1. CI Build and Test (`ci.yml`)

**Triggers:**
- Push to `master`, `main`, or `develop` branches
- Pull requests to `master`, `main`, or `develop` branches

**Jobs:**

#### Test Suite
- Runs on stable, beta, and nightly Rust versions
- Executes `cargo test --release`
- Runs documentation tests
- Caches dependencies for faster builds

**Status:** Runs on matrix of Rust versions for maximum compatibility

#### Lint (Clippy & Fmt)
- Checks code formatting with `rustfmt`
- Runs clippy with warnings-as-errors
- Ensures consistent code style

**Status:** Must pass before merge

#### Security Audit
- Checks for known security vulnerabilities
- Uses `rustsec/audit-check-action`
- Prevents deployment of vulnerable dependencies

**Status:** Must pass before merge

#### Build Debian Package
- Builds for multiple architectures:
  - `amd64` (x86_64)
  - `arm64` (aarch64)
  - `armhf` (armv7)
  - `i386` (i686)
- Compiles Rust binaries with release optimizations
- Uploads artifacts for testing

**Status:** Artifacts retained for 7 days

#### Code Coverage
- Generates code coverage reports
- Uploads to Codecov
- Tracks test coverage over time

**Status:** Optional (doesn't block builds)

#### Dependency Check
- Analyzes dependency tree
- Detects duplicate dependencies
- Helps identify optimization opportunities

**Status:** Informational only

### 2. Release Build (`release.yml`)

**Triggers:**
- When a tag matching `v*` is pushed
- Example: `git tag v0.1.0 && git push --tags`

**Jobs:**

#### Create Release
- Creates a GitHub Release
- Generates automatic release notes
- Prepares upload URLs for artifacts

#### Build DEB Packages
- Builds Debian packages for all architectures
- Creates source tarball
- Uploads `.deb` files to release

**Artifacts per architecture:**
- `rclone-mount-tray_VERSION-1_ARCH.deb` - Debian package

#### Build Binaries
- Builds standalone binaries for all architectures
- Strips debugging symbols
- Generates SHA256 checksums
- Uploads to release

**Artifacts per architecture:**
- `rclone-mount-tray-ARCH` - Standalone binary
- `rclone-mount-tray-ARCH.sha256` - Checksum file

#### Release Notes
- Generates automated changelog
- Summarizes built artifacts

### 3. Debian Package Tests (`debian-tests.yml`)

**Triggers:**
- Push to `master`, `main`, or `develop` with changes to:
  - `debian/` directory
  - `Cargo.toml`
  - `Cargo.lock`
  - `src/` directory

**Jobs:**

#### Build DEB Test
- Builds Debian package
- Verifies file contents
- Extracts and tests package

#### Autopkgtest
- Tests package installation on Ubuntu Jammy and Noble
- Uses LXD containers for isolation
- Validates package functionality

#### Lintian Check
- Runs Debian packaging linter
- Checks for common packaging issues
- Validates `debian/changelog` format

## Workflow Files

### `.github/workflows/ci.yml`
- Main CI pipeline
- Tests on every commit
- 525 lines of workflow configuration

### `.github/workflows/release.yml`
- Release automation
- Triggered by version tags
- 200+ lines of configuration

### `.github/workflows/debian-tests.yml`
- Package-specific tests
- Validates Debian packaging
- 150+ lines of configuration

## Creating a Release

### Step 1: Prepare Release

Update version information:

```bash
# Edit Cargo.toml
# Update version = "0.2.0"

# Update debian/changelog
dch -v 0.2.0-1 "Release notes here"

# Commit changes
git add Cargo.toml debian/changelog
git commit -m "chore: Bump version to 0.2.0"
git push origin master
```

### Step 2: Create Version Tag

```bash
# Create annotated tag
git tag -a v0.2.0 -m "Release version 0.2.0"

# Push tag to trigger release workflow
git push origin v0.2.0
```

### Step 3: Monitor Release Build

Visit: https://github.com/pilakkat1964/rclone-mount-tray/actions

Watch the `Release Build` workflow:
- Check build status for each architecture
- Monitor artifact uploads
- Verify release creation

### Step 4: Verify Release

Visit: https://github.com/pilakkat1964/rclone-mount-tray/releases/tag/v0.2.0

Verify all artifacts are present:
- Debian packages (amd64, arm64, armhf, i386)
- Standalone binaries (amd64, arm64, armhf, i386)
- SHA256 checksums

## Artifact Locations

### CI Build Artifacts
- **Location:** GitHub Actions "Artifacts" section
- **Retention:** 7 days
- **Contents:** Built Debian packages

### Release Artifacts
- **Location:** GitHub Releases page
- **Retention:** Permanent
- **Contents:**
  - Debian packages
  - Standalone binaries
  - Checksums

## Build Status Badge

Add to README:

```markdown
[![CI Build Status](https://github.com/pilakkat1964/rclone-mount-tray/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/pilakkat1964/rclone-mount-tray/actions/workflows/ci.yml)
```

## Supported Architectures

The CI/CD pipeline builds for:

| Architecture | Target Triple | GitHub Runner |
|-------------|---------------|----------------|
| amd64 | x86_64-unknown-linux-gnu | ubuntu-latest |
| arm64 | aarch64-unknown-linux-gnu | ubuntu-latest |
| armhf | armv7-unknown-linux-gnueabihf | ubuntu-latest |
| i386 | i686-unknown-linux-gnu | ubuntu-latest |

## Environment Variables

All workflows use these environment variables:

```yaml
CARGO_TERM_COLOR: always    # Colored Cargo output
RUST_BACKTRACE: 1           # Full error backtraces
```

## Cache Strategy

The CI pipeline caches:

- **Cargo registry** - Downloaded crate metadata
- **Cargo git** - Cloned git dependencies
- **Build target** - Compiled artifacts

Cache keys are based on `Cargo.lock` to ensure consistency.

## Security Considerations

### Secrets

No secrets are currently required. All workflows use public repositories.

If needed, secrets can be added:
```yaml
env:
  TOKEN: ${{ secrets.CUSTOM_TOKEN }}
```

### Permissions

The pipeline uses:
- `GITHUB_TOKEN` for release creation
- `Read-only` access to repository code

### Signed Releases

For production use, consider:
1. Signing artifacts with GPG
2. Creating signed commits
3. Verifying checksums

## Troubleshooting

### Build Failures

**Check logs:**
```bash
# Visit Actions page
https://github.com/pilakkat1964/rclone-mount-tray/actions

# Click failed workflow run
# Expand failed job to see logs
```

### Cargo Cache Issues

**Clear cache:**
1. Visit: https://github.com/pilakkat1964/rclone-mount-tray/settings/actions
2. Click "Manage Actions"
3. Clear cached data if needed

### Dependency Issues

**Update dependencies:**
```bash
cargo update
git add Cargo.lock
git commit -m "chore: Update dependencies"
git push origin master
```

## Common Issues

### Issue: Release workflow doesn't trigger

**Solution:** Ensure tag matches `v*` pattern
```bash
git tag v0.2.0      # Correct
git tag 0.2.0       # Won't trigger (missing 'v')
git tag release-0.2.0  # Won't trigger (wrong prefix)
```

### Issue: Build times are long

**Solution:** Cached builds are faster. First build takes longer.

### Issue: Artifacts not uploaded to release

**Solution:** Check workflow logs for errors. Ensure all build jobs passed.

## Monitoring

### GitHub Actions Dashboard

Monitor workflows at: https://github.com/pilakkat1964/rclone-mount-tray/actions

### Status Checks

Status checks are required for:
- Pull requests: All tests must pass
- Direct pushes: Tests run but don't block

### Build History

View historical builds:
```bash
# Via GitHub CLI
gh run list -w ci.yml -L 10

# Via web interface
https://github.com/pilakkat1964/rclone-mount-tray/actions/workflows/ci.yml
```

## Maintenance

### Update Rust Version

Edit `.github/workflows/ci.yml`:
```yaml
rust: [stable, beta, nightly]  # Update versions as needed
```

### Add New Architectures

Edit build matrices in workflows:
```yaml
strategy:
  matrix:
    include:
      - arch: amd64
        target: x86_64-unknown-linux-gnu
      # Add new architecture here
```

### Modify Test Suite

Edit `src/` or `Cargo.toml`:
```bash
# Tests run automatically on next push
git push origin master
```

## Performance

### Build Times

Typical build times (first build):
- **Test Suite:** 5-10 minutes
- **Debian Build:** 3-5 minutes per architecture
- **Security Audit:** 1-2 minutes
- **Code Coverage:** 3-5 minutes

Subsequent builds use cache (1-2 minutes faster).

### Parallel Execution

All jobs run in parallel by default. Total pipeline time is determined by longest job.

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Cargo Documentation](https://doc.rust-lang.org/cargo/)
- [Debian Packaging Guide](https://www.debian.org/doc/manuals/debmaint-faq/)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
