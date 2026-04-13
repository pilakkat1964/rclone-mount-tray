# Quick Reference Guide

## Common Rust Patterns Used in This Project

### 1. Creating a Result-Returning Function

```rust
use anyhow::{anyhow, Result};

fn load_config() -> Result<String> {
    let path = "/path/to/config";
    
    std::fs::read_to_string(path)
        .map_err(|e| anyhow!("Failed to load config: {}", e))
}

// Usage:
match load_config() {
    Ok(config) => println!("Loaded: {}", config),
    Err(e) => eprintln!("Error: {}", e),
}

// Or use ? operator to propagate:
fn process() -> Result<()> {
    let config = load_config()?;  // Returns on error
    println!("Processing: {}", config);
    Ok(())
}
```

### 2. Enum Pattern Matching

```rust
#[derive(Debug)]
pub enum CloudService {
    GoogleDrive,
    OneDrive,
    Dropbox,
}

// Match all variants
match service {
    CloudService::GoogleDrive => {
        println!("Configuring Google Drive");
    }
    CloudService::OneDrive => {
        println!("Configuring OneDrive");
    }
    CloudService::Dropbox => {
        println!("Configuring Dropbox");
    }
}

// Exhaustive matching catches errors at compile time!
// If you add a new variant, compiler forces you to handle it.
```

### 3. Option and None Handling

```rust
// Option<T> = Some(T) or None

let maybe_name: Option<String> = Some("My Drive".to_string());

// Pattern match
match maybe_name {
    Some(name) => println!("Name: {}", name),
    None => println!("No name set"),
}

// Use unwrap_or for default
let name = maybe_name.unwrap_or_else(|| "Unnamed".to_string());

// Use map to transform
let upper = maybe_name.map(|n| n.to_uppercase());

// Chain operations with ?
fn get_first_remote() -> Option<String> {
    let remotes = vec!["drive1", "drive2"];
    let first = remotes.first()?;  // Returns None if empty
    Some(first.to_string())
}
```

### 4. Shared Mutable State

```rust
use std::sync::{Arc, Mutex};

// Create shared state
let config = Arc::new(Mutex::new(vec![1, 2, 3]));

// Clone for use in closure
let config_clone = config.clone();

button.connect_clicked(move || {
    // Lock the mutex
    if let Ok(mut cfg) = config_clone.lock() {
        cfg.push(4);  // Modify safely
        println!("Config: {:?}", *cfg);
    }
    // Lock released when cfg dropped
});
```

**Important:** 
- `Arc` = Atomic Reference Count (safe to share)
- `Mutex` = Mutual exclusion (safe concurrent access)
- `.lock()` returns `Result<MutexGuard>`
- Lock automatically releases when guard drops

### 5. String Manipulation

```rust
// Create strings
let s1 = String::from("hello");
let s2 = "hello".to_string();
let s3 = format!("hello {}", "world");

// String slices (don't own data)
let slice: &str = "hello";

// Split and parse
let line = "key = value";
if let Some((key, val)) = line.split_once('=') {
    let key = key.trim();
    let val = val.trim();
    println!("{}={}", key, val);
}

// Join collection
let parts = vec!["a", "b", "c"];
let joined = parts.join(", ");  // "a, b, c"

// Replace patterns
let text = "old text";
let replaced = text.replace("old", "new");

// Contains and starts_with
if text.contains("test") {
    println!("Found!");
}

if text.starts_with("test") {
    println!("Starts with test");
}
```

### 6. Iterating and Collecting

```rust
let numbers = vec![1, 2, 3, 4, 5];

// Simple iteration
for num in &numbers {
    println!("{}", num);
}

// Transform with map
let doubled: Vec<i32> = numbers.iter().map(|n| n * 2).collect();

// Filter and map
let evens: Vec<i32> = numbers
    .iter()
    .filter(|n| n % 2 == 0)
    .map(|n| n * 10)
    .collect();

// Find first matching
if let Some(first) = numbers.iter().find(|n| n > &&3) {
    println!("First > 3: {}", first);
}

// Fold/reduce
let sum = numbers.iter().fold(0, |acc, n| acc + n);

// Lines from string
let text = "line1\nline2\nline3";
for line in text.lines() {
    println!("{}", line);
}
```

### 7. File Operations

```rust
use std::fs;
use std::path::Path;

// Read entire file
let content = fs::read_to_string("/path/to/file")?;

// Write entire file (overwrites)
fs::write("/path/to/file", "content")?;

// Check if exists
if Path::new("/path/to/file").exists() {
    println!("File exists");
}

// Create directory
fs::create_dir_all("/path/to/dir")?;

// List directory
for entry in fs::read_dir("/path/to/dir")? {
    let entry = entry?;
    let path = entry.path();
    println!("{:?}", path);
}

// Get home directory
if let Some(home) = dirs::home_dir() {
    println!("Home: {:?}", home);
}

// Get config directory (~/.config)
if let Some(config) = dirs::config_dir() {
    println!("Config: {:?}", config);
}
```

### 8. Running External Commands

```rust
use std::process::Command;

// Simple execution
let output = Command::new("systemctl")
    .args(&["--user", "list-units"])
    .output()?;

if !output.status.success() {
    let error = String::from_utf8_lossy(&output.stderr);
    return Err(anyhow!("Command failed: {}", error));
}

let stdout = String::from_utf8_lossy(&output.stdout);
println!("Output: {}", stdout);

// Parse output
for line in stdout.lines() {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if let Some(service_name) = parts.first() {
        println!("Service: {}", service_name);
    }
}
```

### 9. Type Conversion

```rust
// String to number
let num_str = "42";
let num: i32 = num_str.parse()?;  // Result<i32, ParseIntError>

// Number to string
let num = 42;
let str1 = num.to_string();
let str2 = format!("{}", num);

// Type conversion with as
let byte: u8 = 255;
let int: i32 = byte as i32;

// Convert between Result and Option
let result: Result<i32> = Ok(42);
let option: Option<i32> = result.ok();

let option: Option<i32> = Some(42);
let result: Result<i32, &str> = option.ok_or("No value");
```

### 10. Logging

```rust
use tracing::{info, debug, warn, error, trace};

// Setup (in main)
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();

// Use anywhere
info!("Application started");
debug!("Debug details: {:?}", some_value);
warn!("Warning: this might fail");
error!("Error occurred: {}", err);
trace!("Very detailed trace");

// With structured fields
info!(
    remotes = 5,
    mounts = 3,
    "System state"
);

// Span for tracking function execution
use tracing::instrument;

#[instrument]
fn process_remote(remote: &RemoteConfig) -> Result<()> {
    info!("Processing remote");
    // All logs from here are tagged with this function
    Ok(())
}
```

---

## GTK4 Common Code Snippets

### Creating Widgets

```rust
use gtk4::prelude::*;
use gtk4::*;

// Labels
let label = Label::new(Some("Text"));
label.set_wrap(true);
label.add_css_class("title-1");

// Buttons
let button = Button::with_label("Click Me");
button.add_css_class("suggested-action");
button.set_sensitive(false);  // Disable

// Entry (text input)
let entry = Entry::new();
entry.set_placeholder_text(Some("Enter text"));
let text = entry.text();

// ComboBox
let combo = ComboBoxText::new();
combo.append_text("Option 1");
combo.append_text("Option 2");
combo.set_active(Some(0));

// Containers
let vbox = Box::new(Orientation::Vertical, 12);
let hbox = Box::new(Orientation::Horizontal, 6);

// Scrolling
let scrolled = ScrolledWindow::new();
scrolled.set_child(Some(&content_widget));

// List
let list = ListBox::new();
list.set_selection_mode(SelectionMode::Single);
// Add rows programmatically

// Expander (collapsible section)
let expander = Expander::new(Some("Advanced"));
expander.set_child(Some(&advanced_content));
```

### Layout and Positioning

```rust
// Set size
widget.set_width_request(200);
widget.set_height_request(100);

// Set margins
widget.set_margin_top(12);
widget.set_margin_bottom(12);
widget.set_margin_start(12);
widget.set_margin_end(12);

// Expansion
widget.set_hexpand(true);  // Fill horizontal space
widget.set_vexpand(true);  // Fill vertical space

// Alignment
widget.set_halign(Align::Start);   // Left
widget.set_halign(Align::Center);  // Center
widget.set_halign(Align::End);     // Right
widget.set_valign(Align::Start);
widget.set_valign(Align::Center);
widget.set_valign(Align::End);

// Append to containers
container.append(&widget);
container.remove(&widget);

// Set parent
window.set_child(Some(&widget));
```

### Styling with CSS

```rust
// Define CSS
let css = r#"
button {
    padding: 6px 12px;
    border-radius: 4px;
}

button.destructive-action {
    background-color: #e74c3c;
    color: white;
}

label.title-1 {
    font-size: 28px;
    font-weight: bold;
}

.monospace {
    font-family: monospace;
    font-size: 11px;
}
"#;

// Apply CSS
let provider = CssProvider::new();
provider.load_from_data(css);

if let Some(display) = gdk::Display::default() {
    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

// Apply class to widget
widget.add_css_class("destructive-action");
widget.add_css_class("monospace");

// Remove class
widget.remove_css_class("destructive-action");
```

### Signals and Events

```rust
use gtk4::prelude::*;

// Button click
button.connect_clicked(|btn| {
    println!("Button clicked!");
    btn.set_label("Clicked!");
});

// Entry text changed
entry.connect_changed(|entry| {
    let text = entry.text();
    println!("Text: {}", text);
});

// ComboBox selection changed
combo.connect_changed(|combo| {
    if let Some(text) = combo.active_text() {
        println!("Selected: {}", text);
    }
});

// List selection changed
list.connect_row_selected(|_list, row| {
    if let Some(row) = row {
        println!("Selected row: {}", row.index());
    }
});

// Window close requested
window.connect_close_request(|_| {
    println!("Window closing");
    glib::signal::Inhibit(false)  // Allow close
});

// Key press
widget.connect_key_pressed(|_widget, key, _code, _state| {
    match key {
        gdk::Key::Escape => {
            println!("Escape pressed");
            glib::signal::Inhibit(true)  // Stop propagation
        }
        _ => glib::signal::Inhibit(false),
    }
});
```

### Dialogs

```rust
// Error dialog
let dialog = MessageDialog::new(
    Some(&window),
    gtk4::DialogFlags::MODAL,
    gtk4::MessageType::Error,
    gtk4::ButtonsType::Ok,
    "Error Title",
);
dialog.set_secondary_text(Some("Error details"));
dialog.run_async(|dialog, _| dialog.close());

// Confirmation
let dialog = MessageDialog::new(
    Some(&window),
    gtk4::DialogFlags::MODAL,
    gtk4::MessageType::Question,
    gtk4::ButtonsType::YesNo,
    "Confirm?",
);
dialog.set_secondary_text(Some("Really do this?"));
let response = dialog.run();
if response == ResponseType::Yes as i32 {
    println!("User said yes");
}

// File chooser
let dialog = FileChooserDialog::new(
    Some("Open File"),
    Some(&window),
    gtk4::FileChooserAction::Open,
    &[("Cancel", ResponseType::Cancel as i32), ("Open", ResponseType::Accept as i32)],
);

let response = dialog.run();
if response == ResponseType::Accept as i32 {
    if let Some(file) = dialog.file() {
        if let Some(path) = file.path() {
            println!("Selected: {:?}", path);
        }
    }
}
```

---

## Debian Packaging Quick Start

### Minimal debian/control

```
Source: myapp
Section: utils
Priority: optional
Maintainer: Your Name <you@example.com>
Build-Depends: cargo, rustc
Standards-Version: 4.7.0
Rules-Requires-Root: no

Package: myapp
Architecture: amd64 arm64
Depends: ${misc:Depends}
Description: My application
 Short description (continued)
 with details.
```

### Minimal debian/rules

```make
#!/usr/bin/make -f
%:
	dh $@

override_dh_auto_build:
	cargo build --release --locked

override_dh_auto_test:
	cargo test --release || true

override_dh_auto_clean:
	cargo clean || true
	dh_auto_clean
```

### Build commands

```bash
# Build Debian package
dpkg-buildpackage -b -uc -us

# Build source package
dpkg-buildpackage -S -uc -us

# Install locally
sudo dpkg -i ../myapp_1.0-1_amd64.deb

# Uninstall
sudo dpkg -r myapp

# Check package contents
dpkg -c myapp_1.0-1_amd64.deb

# Get package info
dpkg -I myapp_1.0-1_amd64.deb
```

---

## Testing Patterns

### Unit Test

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_service() {
        let service_str = "drive";
        let service = parse_service_type(service_str);
        assert_eq!(service, CloudService::GoogleDrive);
    }

    #[test]
    fn test_remote_creation() {
        let remote = RemoteConfig::new(
            "test".to_string(),
            CloudService::GoogleDrive,
        );
        assert_eq!(remote.name, "test");
        assert!(!remote.auth_method.is_empty());
    }

    #[test]
    #[should_panic(expected = "Failed")]
    fn test_panic() {
        panic!("Failed");
    }
}
```

### Run tests

```bash
cargo test                      # All tests
cargo test --lib              # Library tests only
cargo test specific_test      # One test
cargo test -- --nocapture    # Show println! output
cargo test -- --test-threads=1  # Single-threaded
```

---

## Troubleshooting

### "Cannot find gtk4 libraries"

```bash
# Install dev packages
sudo apt-get install libgtk-4-dev libadwaita-1-dev

# Set pkg-config path if needed
export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig:$PKG_CONFIG_PATH

# Verify
pkg-config --list-all | grep gtk4
```

### "Value moved into closure"

```rust
// Problem
let value = String::from("test");
button.connect_clicked(|| {
    println!("{}", value);  // ❌ value moved
});

// Solution 1: Clone
let value_clone = value.clone();
button.connect_clicked(move || {
    println!("{}", value_clone);  // ✅ Using clone
});

// Solution 2: Use Arc for complex types
let value = Arc::new(value);
let value_clone = value.clone();
button.connect_clicked(move || {
    println!("{}", value_clone);
});
```

### "Unwrap panicked"

```rust
// Bad
let value = operation().unwrap();  // Panics if error

// Good
match operation() {
    Ok(v) => { /* use v */ }
    Err(e) => eprintln!("Error: {}", e),
}

// Or propagate
let value = operation()?;  // Returns if error
```

### Binary too large

```bash
# Strip debug symbols
cargo build --release
strip target/release/myapp

# Check size before/after
ls -lh target/release/myapp

# Profile size usage
cargo bloat --release
```

---

## Performance Profiling

### Check binary size

```bash
cargo build --release
ls -lh target/release/rclone-config-manager

# Detailed breakdown
cargo bloat --release

# Find largest functions
nm -rS target/release/rclone-config-manager | head -20
```

### Profile runtime

```bash
# Linux perf
sudo perf record ./target/release/myapp
sudo perf report

# Valgrind (memory)
valgrind --leak-check=full ./target/release/myapp

# Time execution
time ./target/release/myapp

# Monitor resources
top
htop
watch -n 1 'ps aux | grep myapp'
```

---

## Git Workflow

```bash
# See what changed
git status
git diff

# Stage changes
git add file.rs
git add .  # All changes

# Commit with message
git commit -m "Fix bug in config parsing"

# View history
git log
git log --oneline
git log -p  # Show diffs

# Create branch
git checkout -b feature/oauth

# Switch branches
git checkout main
git switch feature/oauth

# Merge branch
git merge feature/oauth

# Push to remote
git push origin main
```

---

## Resources

- **Rust Book**: https://doc.rust-lang.org/book/
- **GTK4 Rust**: https://gtk-rs.org/gtk4-rs/
- **Tokio Async**: https://tokio.rs/
- **Error Handling**: https://doc.rust-lang.org/rust-by-example/error.html
- **Debian Guide**: https://www.debian.org/doc/debian-policy/

---

**This guide provides copy-paste ready patterns for common tasks!**
