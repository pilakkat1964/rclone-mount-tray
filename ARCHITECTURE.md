# Visual Architecture & Design Patterns

## System Architecture Overview

### Complete System Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    RClone Mount Management System               │
└─────────────────────────────────────────────────────────────────┘

                              ┌──────────────────────┐
                              │   User Desktop       │
                              │  (GNOME/Plasma/etc)  │
                              └──────────────────────┘
                                      ▲
                    ┌───────────────────┼───────────────────┐
                    │                   │                   │
                    ▼                   ▼                   ▼
            ┌─────────────┐   ┌─────────────────┐  ┌──────────────┐
            │   Tray App  │   │  Config Manager │  │ CLI Commands │
            │   (Rust)    │   │   GUI (GTK4)    │  │  (Bash)      │
            └─────────────┘   └─────────────────┘  └──────────────┘
                    │                   │                   │
                    └───────────────────┬───────────────────┘
                                        ▼
                            ┌──────────────────────┐
                            │  Shared Services    │
                            ├──────────────────────┤
                            │ Config Manager       │
                            │ Auth Handler         │
                            │ Systemd Manager      │
                            └──────────────────────┘
                                        │
                    ┌───────────────────┼───────────────────┐
                    │                   │                   │
                    ▼                   ▼                   ▼
            ┌─────────────┐   ┌─────────────────┐  ┌──────────────┐
            │ rclone.conf │   │ Systemd User    │  │ File System  │
            │             │   │ Services        │  │ (~/.config)  │
            └─────────────┘   └─────────────────┘  └──────────────┘
                    │                   │
                    ▼                   ▼
            ┌─────────────────────────────────┐
            │      Remote Cloud Storage       │
            │  (Google Drive, OneDrive, etc)  │
            └─────────────────────────────────┘
```

---

## Module Dependency Graph

```
                            main.rs
                              │
                ┌─────────────┬┴────────────┬─────────────┐
                │             │            │             │
                ▼             ▼            ▼             ▼
            models/       config/       services/      ui/
            mod.rs        mod.rs        mod.rs         mod.rs
                │             │            │        ┌────┴────┐
                │             │            │        │          │
                │             ▼            │    dialogs.rs  widgets.rs
                │        RemoteConfig      │        │          │
                │        MountConfig   ────┘        └──┬───────┘
                │        CloudService                  │
                │        MountStatus                   │
                │        AuthCred.                     │
                │             │                       │
                └─────────────┴───────────────────────┘
                              │
                    Uses models throughout
```

---

## Data Flow: Adding a Remote

```
User Action: Click "Add Remote" Button
                        │
                        ▼
    ┌──────────────────────────────────┐
    │   AddRemoteDialog::new()         │
    │   - Show dialog window           │
    │   - Present to user              │
    └──────────────────────────────────┘
                        │
                        ▼
    User enters:
    - Name: "My Google Drive"
    - Service: Google Drive
    - Auth: OAuth
                        │
                        ▼
    ┌──────────────────────────────────┐
    │   AddRemoteDialog::run()         │
    │   - Wait for user input          │
    │   - Validate on Accept           │
    │   - Create RemoteConfig struct   │
    └──────────────────────────────────┘
                        │
                        ▼
    RemoteConfig {
      name: "My Google Drive",
      service: GoogleDrive,
      auth_method: "oauth",
      credentials: {...},
      properties: {...}
    }
                        │
                        ▼
    ┌──────────────────────────────────┐
    │   config_manager.lock()          │
    │   .write_remote(&remote)         │
    └──────────────────────────────────┘
                        │
                        ▼
    ┌──────────────────────────────────┐
    │   RcloneConfigManager            │
    │   - Read current rclone.conf     │
    │   - Backup original              │
    │   - Parse sections               │
    │   - Add/update [remote] section  │
    │   - Write back to file           │
    └──────────────────────────────────┘
                        │
                        ▼
    ~/.config/rclone/rclone.conf updated:
    
    [My Google Drive]
    type = drive
    client_id = xxx...
    client_secret = yyy...
                        │
                        ▼
    ┌──────────────────────────────────┐
    │   show_info_dialog()             │
    │   "Remote saved successfully"    │
    └──────────────────────────────────┘
                        │
                        ▼
    ┌──────────────────────────────────┐
    │   refresh_remotes_list()         │
    │   - Parse all remotes from file  │
    │   - Clear list box               │
    │   - Add rows for each remote     │
    └──────────────────────────────────┘
                        │
                        ▼
    UI Updated: New remote visible in list
```

---

## Data Flow: Mounting a Remote

```
User Action: Click "Mount" Button on Remote
                        │
                        ▼
    ┌──────────────────────────────────┐
    │   Mount Creation Dialog          │
    │   - Select remote               │
    │   - Enter mount point (e.g.)    │
    │     /home/user/gdrive           │
    │   - Configure options           │
    └──────────────────────────────────┘
                        │
                        ▼
    MountConfig {
      id: UUID,
      name: "My Google Drive Mount",
      remote_name: "My Google Drive",
      mount_point: "/home/user/gdrive",
      options: MountOptions::default(),
      enabled: true
    }
                        │
                        ▼
    ┌──────────────────────────────────┐
    │   config_manager.write_mount()   │
    │   Persist mount config to file   │
    └──────────────────────────────────┘
                        │
                        ▼
    ┌──────────────────────────────────┐
    │   SystemdManager::generate_      │
    │   service(remote, mount, opts)   │
    │   Creates service file content   │
    └──────────────────────────────────┘
                        ▼
    Service File Content:
    
    [Unit]
    Description=RClone mount for My Google Drive
    After=network-online.target
    
    [Service]
    Type=notify
    ExecStart=/usr/bin/rclone mount \
      "My Google Drive" \
      /home/user/gdrive
    Restart=on-failure
    
    [Install]
    WantedBy=default.target
                        │
                        ▼
    ┌──────────────────────────────────┐
    │   Write service file to:         │
    │   ~/.config/systemd/user/        │
    │   rclone-mount-*.service         │
    └──────────────────────────────────┘
                        │
                        ▼
    ┌──────────────────────────────────┐
    │   SystemdManager::reload_daemon()│
    │   $ systemctl --user             │
    │     daemon-reload                │
    └──────────────────────────────────┘
                        │
                        ▼
    ┌──────────────────────────────────┐
    │   SystemdManager::start_mount()  │
    │   $ systemctl --user start \     │
    │     rclone-mount-*.service       │
    └──────────────────────────────────┘
                        │
                        ▼
    ┌──────────────────────────────────┐
    │   Poll SystemdManager::          │
    │   is_mounted() every 5 seconds   │
    │   Check active status            │
    └──────────────────────────────────┘
                        │
                        ▼
    Mount Status Changes:
    Unmounted → Mounting → Mounted
                        │
                        ▼
    UI Updated:
    - Status indicator changes to green
    - Button text changes to "Unmount"
    - Mount point shows directory listing
```

---

## GTK4 Widget Hierarchy Example

```
ApplicationWindow
│
└── Box (Vertical)              # Main container
    │
    ├── HeaderBar               # Title bar
    │   ├── Label "Remote Configuration Manager"
    │   └── Button "Settings"
    │
    ├── Box (Horizontal)        # Content area
    │   │
    │   ├── StackSidebar        # Left navigation
    │   │   ├── "Remotes"
    │   │   ├── "Mounts"
    │   │   └── "Settings"
    │   │
    │   └── Stack               # Right content area
    │       │
    │       ├── Page: Remotes
    │       │   └── Box (Vertical)
    │       │       ├── Box (Horizontal)
    │       │       │   ├── Label "Cloud Service Remotes"
    │       │       │   └── Button "Add Remote"
    │       │       └── ScrolledWindow
    │       │           └── ListBox
    │       │               ├── ListBoxRow
    │       │               │   └── Box (Horizontal)
    │       │               │       ├── Label "🔵"
    │       │               │       ├── Box (Vertical)
    │       │               │       │   ├── Label "My Drive"
    │       │               │       │   └── Label "Google Drive"
    │       │               │       ├── Button "Edit"
    │       │               │       └── Button "Delete"
    │       │               └── ... (more rows)
    │       │
    │       ├── Page: Mounts
    │       │   └── Box (Vertical)
    │       │       ├── Box (Horizontal)
    │       │       │   ├── Label "Active Mounts"
    │       │       │   └── Button "Add Mount"
    │       │       └── ScrolledWindow
    │       │           └── ListBox
    │       │               └── ... (mount rows)
    │       │
    │       └── Page: Settings
    │           └── Label "Settings coming soon"
    │
    └── StatusBar               # Bottom status
        └── Label "Ready"
```

---

## State Management Pattern

```
┌────────────────────────────────────────┐
│  Application State (Arc<Mutex<T>>)     │
│                                        │
│  ┌──────────────────────────────────┐ │
│  │ ConfigManager                    │ │
│  │ ├── config_path                  │ │
│  │ └── methods: parse(), write()    │ │
│  └──────────────────────────────────┘ │
│                                        │
│  ┌──────────────────────────────────┐ │
│  │ SystemdManager (static methods)  │ │
│  │ ├── service_name()               │ │
│  │ ├── start_mount()                │ │
│  │ └── is_mounted()                 │ │
│  └──────────────────────────────────┘ │
└────────────────────────────────────────┘
           ▲
           │ Arc clone
           │ Shared ownership
           │
    ┌──────┴──────┬──────────┬──────────┐
    │             │          │          │
    ▼             ▼          ▼          ▼
┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐
│ Main UI  │ │ Dialog   │ │ Refresh  │ │  Event   │
│ Closure  │ │ Closure  │ │ Thread   │ │ Handler  │
└──────────┘ └──────────┘ └──────────┘ └──────────┘
    │             │          │          │
    └──────────────┴──────────┴──────────┘
                   │
            Uses lock()
            Accesses data
            Releases lock
```

---

## Event Handling Flow

```
User Interaction
        │
        ▼
┌─────────────────────────────────┐
│  GTK Signal Emitted             │
│  (button-clicked, changed, etc) │
└─────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────┐
│  Signal Handler Closure         │
│  connect_clicked(|btn| { ... }) │
└─────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────┐
│  Acquire Lock                   │
│  config_manager.lock()          │
└─────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────┐
│  Perform Operation              │
│  - Read config                  │
│  - Execute command              │
│  - Update state                 │
└─────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────┐
│  Handle Result                  │
│  - Ok: Update UI                │
│  - Err: Show error dialog       │
└─────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────┐
│  Update UI                      │
│  - Refresh lists                │
│  - Change button state          │
│  - Show feedback                │
└─────────────────────────────────┘
        │
        ▼
┌─────────────────────────────────┐
│  Lock Released                  │
│  (MutexGuard dropped)           │
└─────────────────────────────────┘
```

---

## Error Handling Pattern

```
┌────────────────────────────┐
│  Operation                 │
│  (fallible action)         │
└────────────────────────────┘
            │
            ▼
    ┌──────────────────┐
    │  Result<T, E>    │
    └──────────────────┘
         /        \
        /          \
       ▼            ▼
    ┌──────┐    ┌──────────┐
    │ Ok   │    │ Err      │
    │ (T)  │    │ (E)      │
    └──────┘    └──────────┘
       │            │
       │            ▼
       │    ┌──────────────────┐
       │    │ Propagate with ? │
       │    │ Return early     │
       │    └──────────────────┘
       │            │
       │            ▼
       │    ┌──────────────────┐
       │    │ Log error        │
       │    │ Show to user     │
       │    │ Recover or exit  │
       │    └──────────────────┘
       │
       ▼
    ┌──────────────────┐
    │ Use value T      │
    │ Continue flow    │
    └──────────────────┘
```

### Example: Parse Remotes with Error Handling

```rust
fn refresh_remotes(config_manager: Arc<Mutex<RcloneConfigManager>>) 
    -> Result<Vec<RemoteConfig>> 
{
    // Acquire lock - may fail
    let cm = config_manager
        .lock()
        .map_err(|e| anyhow!("Failed to acquire lock: {}", e))?;
    
    // Parse remotes - may fail
    let remotes = cm
        .parse_remotes()
        .map_err(|e| anyhow!("Failed to parse remotes: {}", e))?;
    
    // Lock released here (cm dropped)
    Ok(remotes)
}

// Usage:
match refresh_remotes(config_manager.clone()) {
    Ok(remotes) => {
        println!("Loaded {} remotes", remotes.len());
        update_ui(&remotes);
    }
    Err(e) => {
        eprintln!("Error: {}", e);
        show_error_dialog("Failed to load remotes", &e.to_string());
    }
}
```

---

## Configuration File Processing

```
Input: rclone.conf file
            │
            ▼
┌──────────────────────────┐
│  Read file to string     │
│  fs::read_to_string()    │
└──────────────────────────┘
            │
            ▼
┌──────────────────────────┐
│  Iterate lines           │
│  content.lines()         │
└──────────────────────────┘
            │
            ▼
┌──────────────────────────┐
│  For each line:          │
│                          │
│  if [section_name]       │
│    ├── Save previous     │
│    └── Start new remote  │
│                          │
│  if key = value          │
│    └── Add property      │
│                          │
│  if empty or comment     │
│    └── Skip              │
└──────────────────────────┘
            │
            ▼
┌──────────────────────────┐
│  Collect RemoteConfigs   │
│  Into Vec<RemoteConfig>  │
└──────────────────────────┘
            │
            ▼
Output: Parsed remotes ready for UI
```

---

## Systemd Service Lifecycle

```
User creates mount in GUI
            │
            ▼
┌──────────────────────────┐
│  GenerateService()       │
│  Create INI content      │
└──────────────────────────┘
            │
            ▼
┌──────────────────────────┐
│  Write to file           │
│  ~/.config/systemd/user/ │
│  rclone-mount-*.service  │
└──────────────────────────┘
            │
            ▼
┌──────────────────────────┐
│  ReloadDaemon()          │
│  systemctl --user        │
│  daemon-reload           │
└──────────────────────────┘
            │
            ▼
┌──────────────────────────┐
│  start_mount()           │
│  systemctl --user        │
│  start SERVICE_NAME      │
└──────────────────────────┘
            │
            ▼
        ┌───────────────────────────┐
        │   Service Active States   │
        ├───────────────────────────┤
        │ inactive   → starting...  │
        │ activating → active       │
        │ active     → running      │
        │ deactivating → inactive   │
        │ failed     → error        │
        └───────────────────────────┘
            │
            ▼
┌──────────────────────────┐
│  is_mounted()            │
│  systemctl --user        │
│  is-active SERVICE_NAME  │
│  Returns: true/false     │
└──────────────────────────┘
            │
            ▼
┌──────────────────────────┐
│  Update UI Status        │
│  - Green checkmark       │
│  - Mount size display    │
│  - Activity indicator    │
└──────────────────────────┘
            │
            ▼
┌──────────────────────────┐
│  User clicks "Unmount"   │
│        │                 │
│        ▼                 │
│  stop_mount()            │
│  systemctl --user        │
│  stop SERVICE_NAME       │
│        │                 │
│        ▼                 │
│  disable_service()       │
│  (optional)              │
└──────────────────────────┘
            │
            ▼
Service removed from active services
```

---

## Ownership and Borrowing in Event Handlers

```
// Problem: Simple capture doesn't work

let config = String::from("data");
button.connect_clicked(|| {
    // ❌ Error: value moved
    println!("{}", config);
});


// Solution 1: Clone for simple types

let config = String::from("data");
let config_clone = config.clone();
button.connect_clicked(move || {
    // ✅ Works: using clone
    println!("{}", config_clone);
});


// Solution 2: Arc<Mutex<T>> for shared state

let config = Arc::new(Mutex::new(vec![1, 2, 3]));
let config_clone = config.clone();
button.connect_clicked(move || {
    // ✅ Works: using cloned Arc
    if let Ok(cfg) = config_clone.lock() {
        println!("{:?}", *cfg);
    }
});
// Lock released when cfg dropped
```

---

## Thread Safety & Lock Contention

```
UI Thread                Debug Thread        Config Manager
    │                         │                    │
    ├─ Button clicked         │                    │
    │  Acquire lock ──────────────────────────►   │
    │  │                       │                    │
    │  │  Read config                              │
    │  │  Lock held ──────────────────────────────►│
    │  │                       │                    │
    │  │  (Debug checks status)                     │
    │  │  Waits for lock ──────────┬────────────────┤ (Busy)
    │  │                           │                │
    │  │  Release lock ────────────┼──────────────►│
    │  │                           │                │
    │  │  (Debug acquires lock)    │                │
    │  │  Status update ◄──────────┼────────────────┤ (Success)
    │  │                           │                │
    │  Update UI                   │                │
    │                              │                │
    ▼                              ▼                ▼

Goal: Minimize lock duration
✅ Good: Lock only when accessing shared data
❌ Bad: Hold lock during UI updates or I/O
```

---

## Authentication Flow (OAuth)

```
User clicks "Configure" → OAuth
            │
            ▼
┌──────────────────────────────────┐
│  OAuthDialog::new()              │
│  Show "Click to authenticate"    │
└──────────────────────────────────┘
            │
            ▼
┌──────────────────────────────────┐
│  User clicks "Start Auth"        │
│  OAuthDialog::run()              │
└──────────────────────────────────┘
            │
            ▼
┌──────────────────────────────────┐
│  Open browser to OAuth provider  │
│  https://accounts.google.com/    │
│  ?scope=drive.readonly           │
│  &client_id=...                  │
│  &redirect_uri=http://localhost  │
└──────────────────────────────────┘
            │
            ▼
    ┌───────────────────────┐
    │  User logs in         │
    │  Approves permissions │
    └───────────────────────┘
            │
            ▼
┌──────────────────────────────────┐
│  Browser redirects to:           │
│  http://localhost:8000           │
│  ?code=AUTH_CODE                 │
│  &state=STATE_TOKEN              │
└──────────────────────────────────┘
            │
            ▼
┌──────────────────────────────────┐
│  Local server receives code      │
│  Extract AUTH_CODE               │
└──────────────────────────────────┘
            │
            ▼
┌──────────────────────────────────┐
│  Exchange code for tokens        │
│  POST to https://oauth2.googleapis.com/token
│  {                               │
│    code: AUTH_CODE,              │
│    client_id: ...,               │
│    client_secret: ...            │
│  }                               │
└──────────────────────────────────┘
            │
            ▼
┌──────────────────────────────────┐
│  Receive tokens:                 │
│  {                               │
│    access_token: "ya29...",       │
│    refresh_token: "1//...",       │
│    expires_in: 3599              │
│  }                               │
└──────────────────────────────────┘
            │
            ▼
┌──────────────────────────────────┐
│  Store credentials               │
│  AuthCredentials {               │
│    access_token: Some(...),      │
│    refresh_token: Some(...),     │
│    token_expiry: Some("2024...") │
│  }                               │
└──────────────────────────────────┘
            │
            ▼
┌──────────────────────────────────┐
│  Save to rclone.conf             │
│  [remote-name]                   │
│  type = drive                    │
│  token = {"access_token": "..."}│
└──────────────────────────────────┘
            │
            ▼
     Success: Ready to mount
```

---

## Package Building Pipeline

```
Source Code
    │
    ├─ Cargo.toml
    ├─ src/
    ├─ debian/
    └─ README.md
            │
            ▼
    ┌──────────────────────┐
    │  dpkg-buildpackage   │
    │  -B (build binary)   │
    └──────────────────────┘
            │
            ▼
    ┌──────────────────────┐
    │  debian/rules        │
    │  override_dh_auto    │
    │  _build              │
    └──────────────────────┘
            │
            ▼
    ┌──────────────────────┐
    │  cargo build         │
    │  --release --locked  │
    └──────────────────────┘
            │
            ▼
    ┌──────────────────────┐
    │  Binary ready:       │
    │  target/release/     │
    │  rclone-config-mgr   │
    └──────────────────────┘
            │
            ▼
    ┌──────────────────────┐
    │  debian/rules        │
    │  override_dh_auto    │
    │  _install            │
    └──────────────────────┘
            │
            ▼
    ┌──────────────────────┐
    │  Copy to staging:    │
    │  debian/install file │
    │  Desktop entry       │
    │  Documentation       │
    └──────────────────────┘
            │
            ▼
    ┌──────────────────────┐
    │  Create DEBIAN/      │
    │  control, md5sums    │
    │  postinst, postrm    │
    └──────────────────────┘
            │
            ▼
    ┌──────────────────────┐
    │  Package as .deb     │
    │  ar + tar formats    │
    │  metadata + files    │
    └──────────────────────┘
            │
            ▼
    Output: .deb package
    rclone-config-manager_0.1.0-1_amd64.deb
```

---

## Testing Strategy

```
Unit Tests
    ├─ Models
    │   └─ CloudService parsing
    │   └─ RemoteConfig creation
    │   └─ MountConfig defaults
    │
    ├─ Config Manager
    │   └─ Parse remotes from INI
    │   └─ Write remote to file
    │   └─ Remove remote from file
    │
    ├─ Auth
    │   └─ Token validation
    │   └─ Credential storage
    │
    └─ Services
        └─ Service name generation
        └─ Command execution mocking


Integration Tests
    ├─ End-to-end flows
    │   └─ Add remote → Write config
    │   └─ Create mount → Start service
    │   └─ Stop mount → Service gone
    │
    └─ File system operations
        └─ Config file creation
        └─ Backup on write


System Tests
    ├─ Real systemd interaction
    ├─ Actual rclone mounting
    └─ Debian package installation
```

---

## Performance Considerations

### Memory Usage Profile

```
Baseline (idle): ~50-100 MB
    │
    ├─ Binary code: ~10 MB
    ├─ GTK4/libadwaita: ~30 MB
    ├─ Rust runtime: ~5 MB
    └─ Config in memory: ~1 MB

With 10 remotes: +5-10 MB
    └─ Each RemoteConfig ≈ 0.5-1 KB

With 10 active mounts: +5-10 MB
    └─ Status polling, systemd queries

Peak (all UI open): ~150-200 MB
    └─ All dialogs, lists, history
```

### CPU Usage

```
Idle state: <1% CPU
    └─ Waiting for events

Status refresh (5s cycle): 2-5% CPU spike
    ├─ Systemd queries
    ├─ Config file check
    └─ UI update

User interaction: <5% CPU
    ├─ Dialog rendering
    ├─ File I/O
    └─ Config updates
```

---

## Deployment Checklist

```
Code Quality
  ✅ cargo clippy (warnings)
  ✅ cargo fmt (formatting)
  ✅ cargo test (all tests pass)
  ✅ cargo build --release (no errors)

Security
  ✅ No unwrap() in production code
  ✅ Error handling comprehensive
  ✅ No hardcoded secrets
  ✅ Input validation on all external data

Performance
  ✅ Binary size reasonable
  ✅ Memory usage acceptable
  ✅ No memory leaks (valgrind clean)
  ✅ Startup time < 2 seconds

Distribution
  ✅ Debian package builds
  ✅ Desktop entry works
  ✅ Dependencies documented
  ✅ README complete
  ✅ License headers present

Testing
  ✅ Unit tests pass
  ✅ Integration tests pass
  ✅ Manual testing on target system
  ✅ Package installation works
```

---

**These diagrams and patterns complement the TUTORIAL.md document for visual learners!**
