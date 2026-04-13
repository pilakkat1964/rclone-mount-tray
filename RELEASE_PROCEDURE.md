# Release Checklist and Procedures

This guide provides step-by-step instructions for creating releases of rclone-mount-tray.

## Pre-Release Checklist

### 1. Version Planning

- [ ] Decide on version number (follow [Semantic Versioning](https://semver.org/))
  - Major.Minor.Patch (e.g., 0.2.0)
  - Increment MAJOR for incompatible changes
  - Increment MINOR for new features
  - Increment PATCH for bug fixes

### 2. Code Review

- [ ] All PRs reviewed and merged to `master`
- [ ] All CI checks passing
- [ ] Security audit passed
- [ ] Code coverage acceptable (>70% recommended)
- [ ] No known issues or TODOs remaining

### 3. Documentation Updates

- [ ] Update `CHANGELOG.md` with all changes
- [ ] Update `README.md` with new features/fixes
- [ ] Update API documentation if applicable
- [ ] Review all markdown files for typos
- [ ] Update installation instructions if needed

### 4. Testing

- [ ] Run full test suite locally: `cargo test --all`
- [ ] Test Debian package build locally: `./scripts/build-deb.sh`
- [ ] Manual testing on target Ubuntu/Debian versions
- [ ] Test systemd service integration
- [ ] Test on at least one arm64/armhf system if possible

### 5. Dependency Updates

- [ ] Run `cargo update` and review updates
- [ ] Check for security vulnerabilities
- [ ] Commit lock file: `git add Cargo.lock && git commit -m "chore: Update dependencies"`
- [ ] Run security audit: `cargo audit`

## Release Process

### Step 1: Update Version Files

**1.1 Update Cargo.toml:**
```bash
# Edit Cargo.toml
# Change: version = "X.Y.Z"

# Find the line:
# [package]
# version = "0.1.0"
#
# Update to new version:
# version = "0.2.0"
```

**1.2 Update debian/changelog:**
```bash
# Use dch tool to update changelog
dch -v 0.2.0-1 "Release notes - first line"

# Add detailed changelog entries
dch -a "Added new feature X"
dch -a "Fixed bug Y"
dch -a "Improved performance of Z"

# Or manually edit debian/changelog
# Format: package (version) distribution; urgency=medium
#         - bullet points for changes
#
#  -- Maintainer <email>  Date
```

**1.3 Verify version updates:**
```bash
grep "^version" Cargo.toml
head -5 debian/changelog
```

### Step 2: Commit Changes

```bash
# Stage changes
git add Cargo.toml debian/changelog

# Commit with meaningful message
git commit -m "chore: Release version 0.2.0

- Update Cargo.toml version
- Update debian/changelog with release notes
- Release notes for 0.2.0"

# Push to remote
git push origin master
```

### Step 3: Create Git Tag

```bash
# Create annotated tag (recommended)
git tag -a v0.2.0 -m "Release version 0.2.0

Highlights:
- New feature: X
- Bug fix: Y
- Performance improvement: Z

See CHANGELOG.md for full details."

# Or simple tag
git tag -s v0.2.0  # With GPG signing (if configured)

# Verify tag
git show v0.2.0
```

### Step 4: Push Tag

```bash
# Push single tag
git push origin v0.2.0

# Or push all tags
git push origin --tags

# Verify push
git ls-remote origin v0.2.0
```

### Step 5: Monitor Release Build

**Wait for GitHub Actions workflow to complete:**

1. Visit: https://github.com/pilakkat1964/rclone-mount-tray/actions
2. Watch "Release Build" workflow
3. Monitor these jobs:
   - Create Release ✓
   - Build DEB Package (amd64, arm64, armhf, i386) ✓
   - Build Binaries (amd64, arm64, armhf, i386) ✓

**Expected output:**
- GitHub Release created automatically
- Debian packages (.deb) uploaded
- Standalone binaries uploaded
- SHA256 checksums provided

### Step 6: Verify Release

```bash
# Check release on GitHub
gh release view v0.2.0

# List release assets
gh release view v0.2.0 --json assets

# Download and verify
gh release download v0.2.0 -D /tmp/release-test
```

**Verify all artifacts present:**
```
rclone-mount-tray_0.2.0-1_amd64.deb
rclone-mount-tray_0.2.0-1_arm64.deb
rclone-mount-tray_0.2.0-1_armhf.deb
rclone-mount-tray_0.2.0-1_i386.deb
rclone-mount-tray-amd64
rclone-mount-tray-amd64.sha256
rclone-mount-tray-arm64
rclone-mount-tray-arm64.sha256
rclone-mount-tray-armhf
rclone-mount-tray-armhf.sha256
rclone-mount-tray-i386
rclone-mount-tray-i386.sha256
```

### Step 7: Verify Artifacts

```bash
# Verify DEB package
dpkg-deb -c rclone-mount-tray_0.2.0-1_amd64.deb

# Verify checksums
sha256sum -c rclone-mount-tray-amd64.sha256
sha256sum -c rclone-mount-tray-arm64.sha256

# Test DEB installation (in container/VM)
sudo dpkg -i rclone-mount-tray_0.2.0-1_amd64.deb
systemctl --user status rclone-mount-tray
```

### Step 8: Post-Release

```bash
# Create next development version (optional)
# Update Cargo.toml to 0.2.1-dev or next planned version

# Announce release
# - Update website
# - Send to mailing list (if applicable)
# - Post on social media (if applicable)

# Monitor for issues
# - Watch GitHub issues
# - Monitor downloads
# - Collect feedback
```

## Hotfix Release

For urgent bug fixes in released versions:

### Quick Process (same version bump pattern)

1. Create branch from release tag:
   ```bash
   git checkout -b hotfix/0.2.1 v0.2.0
   ```

2. Apply fix:
   ```bash
   # Make code changes
   git add src/...
   git commit -m "fix: Critical bug in feature X"
   ```

3. Update version:
   ```bash
   # Cargo.toml: 0.2.0 → 0.2.1
   # debian/changelog: Add hotfix entry
   ```

4. Merge and tag:
   ```bash
   git checkout master
   git merge hotfix/0.2.1
   git tag -a v0.2.1 -m "Hotfix release"
   git push origin master v0.2.1
   ```

## Version Tagging Convention

### Format

- Semantic versioning: `vMAJOR.MINOR.PATCH`
- Examples: `v0.1.0`, `v1.0.0`, `v2.3.14`

### Rules

- Always use lowercase 'v' prefix
- Match version in `Cargo.toml`
- Annotated tags (preferred): `git tag -a v0.1.0 -m "Release v0.1.0"`
- Signed tags (recommended): `git tag -s v0.1.0`

### Pre-release Tags

For beta/RC releases:
- `v0.1.0-beta.1`
- `v0.1.0-rc.1`
- `v0.1.0-alpha.1`

## Distribution Channels

### GitHub Releases

**Primary distribution channel**
- Direct download from: https://github.com/pilakkat1964/rclone-mount-tray/releases
- Always available for all releases

### System Package Managers

**Future: Ubuntu PPA**
```bash
# Setup for distribution to Ubuntu Launchpad PPA
# See PPA_SETUP.md for details
```

**Future: Debian Repository**
```bash
# Setup for distribution to Debian official repositories
# See DEBIAN_REPO_SETUP.md for details
```

## Verification Procedures

### Package Integrity

```bash
# Verify signature (if signed)
git tag -v v0.2.0

# Verify checksum
sha256sum -c rclone-mount-tray-amd64.sha256

# Compare with GitHub release
curl -s https://api.github.com/repos/pilakkat1964/rclone-mount-tray/releases/tags/v0.2.0
```

### Installation Test

```bash
# In clean Ubuntu 24.04 environment
sudo apt update
sudo apt install ./rclone-mount-tray_0.2.0-1_amd64.deb

# Verify installation
which rclone-mount-tray
systemctl --user status rclone-mount-tray

# Test application
rclone-mount-tray --help
```

## Rollback Procedure

If a release has critical issues:

### Immediate Actions

1. **Disable the release:**
   ```bash
   gh release delete v0.2.0  # Mark as draft
   ```

2. **Notify users:**
   - Post security advisory
   - Recommend holding release

3. **Create hotfix:**
   ```bash
   # Same process as hotfix release
   git checkout -b revert v0.2.0
   # Fix the issue
   git tag -a v0.2.1 -m "Hotfix release - critical issue"
   git push origin v0.2.1
   ```

### Post-Incident

- [ ] Document root cause
- [ ] Add regression tests
- [ ] Review release process
- [ ] Update documentation if needed

## Troubleshooting

### Release Build Failed

**Check logs:**
```bash
gh run list --workflow=release.yml --limit 1
gh run view <run-id> --log
```

**Common causes:**
- Cargo.lock out of sync
- Dependency compilation error
- Insufficient disk space

### Missing Artifacts

**Verify release creation:**
```bash
gh release view v0.2.0 --json assets
```

**If missing, manually upload:**
```bash
gh release upload v0.2.0 ./rclone-mount-tray_0.2.0-1_amd64.deb
gh release upload v0.2.0 ./rclone-mount-tray-amd64.sha256
```

### Version Mismatch

**Verify all files updated:**
```bash
grep -r "0.2.0" Cargo.toml debian/changelog .github/workflows/
```

## Release Notes Template

Create release notes with this structure:

```markdown
## Version 0.2.0 - YYYY-MM-DD

### New Features
- Feature 1: Description
- Feature 2: Description

### Bug Fixes
- Bug 1: Description
- Bug 2: Description

### Improvements
- Improvement 1: Description
- Improvement 2: Description

### Dependencies
- Updated dependency X to Y.Z

### Installation

**Ubuntu/Debian:**
```bash
sudo apt install ./rclone-mount-tray_0.2.0-1_amd64.deb
```

**Other architectures:**
- rclone-mount-tray_0.2.0-1_arm64.deb
- rclone-mount-tray_0.2.0-1_armhf.deb
- rclone-mount-tray_0.2.0-1_i386.deb

### Checksums

| File | SHA256 |
|------|--------|
| rclone-mount-tray-amd64 | `hash...` |
| rclone-mount-tray-arm64 | `hash...` |

### Contributors

- @contributor1 - Feature 1
- @contributor2 - Bug fix 1
```

## Frequency

**Recommended release schedule:**
- Patch releases (0.1.x): As needed for bugs
- Minor releases (0.x.0): Monthly for features
- Major releases (x.0.0): Annually or for breaking changes

## Success Metrics

- [ ] All CI checks pass
- [ ] All artifacts generated
- [ ] Release accessible on GitHub
- [ ] Package installs without errors
- [ ] Application starts and functions
