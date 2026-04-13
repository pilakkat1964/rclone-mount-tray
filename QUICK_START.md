# RClone Mount Manager - Quick Start Guide

## 🚀 5-Minute Quick Start

### What You Now Have

1. **Bash Mount Manager** - Manual mount/unmount via CLI
2. **Rust System Tray Applet** - Visual status monitoring
3. **Systemd Services** - Automated mount execution
4. **Complete Documentation** - Full reference guides

### Quick Commands

#### Mount a Drive (Bash Script)
```bash
rclone-mount gdrive_pilakkat
```

#### Check All Mount Status (Bash Script)
```bash
rclone-status
```

#### Unmount a Drive (Bash Script)
```bash
rclone-unmount gdrive_goofybits
```

#### Start System Tray Applet (Rust App)
```bash
rclone-mount-tray
```

#### Enable Auto-Start (Systemd)
```bash
systemctl --user enable rclone-mount-tray.service
systemctl --user start rclone-mount-tray.service
```

#### View Applet Logs
```bash
journalctl --user -u rclone-mount-tray.service -f
```

## 📋 Comparison: Bash vs. Rust

| Task | Bash Script | Rust Applet |
|------|-------------|------------|
| **Mount drive** | `rclone-mount gdrive_pilakkat` | Via menu (when implemented) |
| **Check status** | `rclone-status` | Always visible in tray |
| **Unmount drive** | `rclone-unmount gdrive_goofybits` | Via menu (when implemented) |
| **Visual feedback** | Command output | Tray icon indicator |
| **Auto-updates** | Manual refresh | Every 5 seconds |
| **Startup** | Per-session | Systemd service |

## 🎯 Usage Scenarios

### Scenario 1: Quick Access (Bash Script)
```bash
# Need to access a specific drive right now?
rclone-mount gdrive_pilakkat
# Now you can: ls ~/gdrive_pilakkat/
rclone-unmount gdrive_pilakkat  # Done using it
```

### Scenario 2: Monitor Status (Rust Applet)
```bash
# Run the applet
rclone-mount-tray &

# Now you can see in your system tray:
# ✓ All mounted
# ◐ Partial mount
# ✗ None mounted
```

### Scenario 3: Scripted Operations
```bash
#!/bin/bash
# Script that needs Google Drive access

rclone-mount gdrive_pilakkat
# Do work with mounted drive
cp ~/gdrive_pilakkat/important.pdf /tmp/
rclone-unmount gdrive_pilakkat
```

## 📁 Where Everything Is

| Component | Location | Type |
|-----------|----------|------|
| **Bash Manager** | `~/.local/bin/rclone-mount-manager.sh` | Bash script |
| **Rust Applet** | `~/.local/bin/rclone-mount-tray` | Binary |
| **Systemd Services** | `~/.config/systemd/user/rclone-gdrive-*.service` | Systemd |
| **Source Code** | `~/workspace/rclone-mount-tray/` | Rust project |
| **Documentation** | See below | Various |

## 📚 Documentation Files

1. **RCLONE_ONDEMAND_GUIDE.md** - On-demand mounting overview
2. **README.md** (bash) - Bash script comprehensive guide
3. **README.md** (rust) - Rust applet user guide
4. **RUST_LEARNING_GUIDE.md** - Rust development guide
5. **PROJECT_SUMMARY.md** - Complete project overview
6. **This file** - Quick start reference

## 🔄 How It All Works Together

```
┌─────────────────────────────────────────┐
│         Your Computer                   │
├─────────────────────────────────────────┤
│                                         │
│  When you run: rclone-mount pilakkat   │
│  ↓                                      │
│  [Bash Script]                          │
│    ↓                                    │
│    Calls: systemctl --user start ...   │
│    ↓                                    │
│    [Systemd Service]                    │
│    ├─ Executes rclone command          │
│    ├─ Mounts at ~/gdrive_pilakkat      │
│    └─ Sets up FUSE filesystem          │
│    ↓                                    │
│    [Rust Applet - running in background]
│    ├─ Detects change within 5 sec      │
│    ├─ Reads /proc/mounts               │
│    ├─ Updates system tray icon         │
│    └─ Shows new mount status           │
│                                         │
│  Result: Files accessible!              │
│  ~/gdrive_pilakkat/ ← Now mounted      │
│                                         │
└─────────────────────────────────────────┘
```

## 🎮 Interactive Example Walkthrough

### Step 1: Check Current Status
```bash
$ rclone-status
RClone Mount Status:

✗ gdrive_pilakkat
  Location: /home/user/gdrive_pilakkat (not mounted)

✗ gdrive_goofybits
  Location: /home/user/gdrive_goofybits (not mounted)
```

### Step 2: Start the Applet
```bash
$ rclone-mount-tray &
# [Rust applet starts in background]
# Tray icon appears showing: ✗ (unmounted)
```

### Step 3: Mount a Drive
```bash
$ rclone-mount gdrive_pilakkat
Mounting gdrive_pilakkat to /home/user/gdrive_pilakkat...
✓ Successfully mounted: gdrive_pilakkat
```

### Step 4: Check Status (Multiple Ways)
```bash
# Via bash script
$ rclone-status
✓ gdrive_pilakkat - 4.8G

# Check directly
$ ls ~/gdrive_pilakkat/
folder1/ folder2/ document.pdf

# Rust applet automatically updated
# Tray icon now shows: ◐ (partial mount)
```

### Step 5: Unmount When Done
```bash
$ rclone-unmount gdrive_pilakkat
Unmounting gdrive_pilakkat...
✓ Successfully unmounted: gdrive_pilakkat

# Tray icon updates automatically: ✗
```

## 🧪 Testing Your Setup

### Test 1: Verify Bash Script Works
```bash
rclone-mount gdrive_pilakkat
# Should see: ✓ Successfully mounted
mountpoint -q ~/gdrive_pilakkat && echo "Confirmed mounted"
rclone-unmount gdrive_pilakkat
# Should see: ✓ Successfully unmounted
```

### Test 2: Verify Rust Applet Runs
```bash
rclone-mount-tray &
sleep 2
ps aux | grep rclone-mount-tray
# Should show running process
kill %1
```

### Test 3: Verify Systemd Service
```bash
systemctl --user status rclone-gdrive-gdrive_pilakkat.service
# Should show: enabled, running (if auto-mounted)
systemctl --user start rclone-gdrive-gdrive_pilakkat.service
mountpoint ~/gdrive_pilakkat && echo "OK"
systemctl --user stop rclone-gdrive-gdrive_pilakkat.service
```

## ⚡ Pro Tips

### Tip 1: Mount All Drives at Once
```bash
rclone-mount-all  # Bash script
```

### Tip 2: Unmount Everything
```bash
rclone-unmount-all  # Bash script
```

### Tip 3: Watch Applet Logs in Real-Time
```bash
journalctl --user -u rclone-mount-tray.service -f
```

### Tip 4: Find a Mount Point
```bash
rclone-status | grep gdrive
# Shows all configured mounts
```

### Tip 5: Rebuild Rust Applet
```bash
cd ~/workspace/rclone-mount-tray
cargo build --release
cp target/release/rclone-mount-tray ~/.local/bin/
systemctl --user restart rclone-mount-tray.service
```

## 🐛 Troubleshooting

### Issue: Bash Functions Not Available
```bash
# Solution: Source the configuration
source ~/.local/bin/rclone-mount-manager.sh
rclone-help
```

### Issue: Applet Not Starting
```bash
# Check if running
pgrep -f rclone-mount-tray

# Try running manually
~/.local/bin/rclone-mount-tray

# Check systemd status
systemctl --user status rclone-mount-tray.service

# View recent logs
journalctl --user -u rclone-mount-tray.service -n 20
```

### Issue: Mount Fails
```bash
# Check if service exists
systemctl --user list-units | grep rclone

# Try starting service directly
systemctl --user start rclone-gdrive-gdrive_pilakkat.service

# Check systemd service logs
journalctl --user -u rclone-gdrive-gdrive_pilakkat.service -n 10
```

### Issue: Applet Not Updating Status
```bash
# Restart the applet
systemctl --user restart rclone-mount-tray.service

# Or kill and restart
pkill -f rclone-mount-tray
sleep 1
rclone-mount-tray &
```

## 📞 Getting Help

### For Bash Script Issues
See: `~/.local/bin/rclone-mount-manager.sh --help`
Or: `~/workspace/rclone/RCLONE_ONDEMAND_GUIDE.md`

### For Rust Applet Issues
See: `~/workspace/rclone-mount-tray/README.md`
Or: `~/workspace/rclone-mount-tray/RUST_LEARNING_GUIDE.md`

### For System Issues
```bash
# Check rclone configuration
rclone listremotes

# Check systemd user services
systemctl --user list-units

# Check available mounts
mount | grep rclone

# Check /proc/mounts
cat /proc/mounts | grep rclone
```

## 🎓 Next Steps

1. **Learn the Bash Script** - Read `RCLONE_ONDEMAND_GUIDE.md`
2. **Understand Rust Applet** - Read `RUST_LEARNING_GUIDE.md`
3. **Customize Configuration** - Edit `~/.config/rclone/rclone.conf`
4. **Explore Source Code** - Check `~/workspace/rclone-mount-tray/src/`
5. **Extend Features** - Consider future enhancements

## 🎯 Common Workflows

### Workflow A: Occasional Access
```bash
# Only mount when needed
rclone-mount gdrive_pilakkat
# Use the drive...
rclone-unmount gdrive_pilakkat
```

### Workflow B: Work Session
```bash
# Mount all drives at start of session
rclone-mount-all

# Run Rust applet to monitor
rclone-mount-tray &

# Work with mounted drives
# ...

# Unmount all when done
rclone-unmount-all
```

### Workflow C: Automated Process
```bash
#!/bin/bash
# In a cron job or script

rclone-mount gdrive_pilakkat

# Do automated work
find ~/gdrive_pilakkat -name "*.pdf" -exec process {} \;

rclone-unmount gdrive_pilakkat
```

---

**Ready to use!** Your rclone on-demand mount system is fully operational.

Start with: `rclone-status` (Bash) or `rclone-mount-tray` (Rust)
