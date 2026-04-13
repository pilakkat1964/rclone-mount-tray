# GTK4 Event Handler Implementation Guide

## Overview

This guide explains how event handlers are implemented in the RClone Config Manager GTK4 GUI application. It covers the patterns used for button clicks, dialog interactions, and list selections.

## Key Concepts

### 1. Signal-Based Event System

GTK4 uses signals to notify about user interactions. Widgets emit signals when events occur:
- `connect_clicked()` - Button click events
- `connect_row_activated()` - ListBox row activation
- `connect_changed()` - ComboBox/Entry text changes
- `connect_toggled()` - ToggleButton state changes

### 2. Closure Capture with `move`

All GTK4 signal handlers use Rust closures. To access variables from the enclosing scope, use the `move` keyword:

```rust
let config_manager = Arc::clone(&self.config_manager);
let window_clone = window.clone();

button.connect_clicked(move |_| {
    // Closure captures config_manager and window_clone
    // The `_` parameter is the button that emitted the signal
});
```

### 3. Thread Safety with Arc<Mutex<T>>

For shared mutable state across event handlers, use `Arc<Mutex<T>>`:

```rust
let config_manager = Arc::new(Mutex::new(RcloneConfigManager::new()?));

// Clone for each event handler
let config_manager_clone = Arc::clone(&config_manager);

button.connect_clicked(move |_| {
    if let Ok(mut cm) = config_manager_clone.lock() {
        // Use cm here
    }
});
```

## Event Handler Patterns

### Pattern 1: Button Click Handler with Dialog

Used for "Add" buttons that open configuration dialogs:

```rust
let config_manager = Arc::clone(&self.config_manager);
let window_clone = window.clone();

add_remote_button.connect_clicked(move |_| {
    tracing::info!("Add Remote button clicked");
    
    // Create and display dialog
    let dialog = AddRemoteDialog::new(&window_clone, None);
    
    // Run dialog and wait for user response
    if let Some(remote_config) = dialog.run() {
        // Process the result
        if let Ok(mut cm) = config_manager.lock() {
            match cm.add_remote(remote_config.clone()) {
                Ok(_) => tracing::info!("Remote added"),
                Err(e) => {
                    widgets::show_error_dialog(
                        &window_clone,
                        "Error",
                        &format!("Failed: {}", e),
                    );
                }
            }
        }
    }
});
```

**Key Points:**
- Clone Arc references with `Arc::clone()` before the move
- Clone widgets with `.clone()` before the move
- Use `dialog.run()` to show modal dialog and wait for response
- Always handle errors with user feedback

### Pattern 2: List Row Activation Handler

Used when clicking on a row to edit its contents:

```rust
let config_manager_clone = Arc::clone(&self.config_manager);
let window_clone = window.clone();

list_box.connect_row_activated(move |list, row| {
    // Get the row's index
    if let Some(index) = list.index_of(row) {
        if let Ok(cm) = config_manager_clone.lock() {
            // Load data at this index
            match cm.parse_remotes() {
                Ok(remotes) => {
                    if let Some(remote) = remotes.get(index as usize) {
                        // Show edit dialog with pre-filled data
                        let dialog = AddRemoteDialog::new(
                            &window_clone,
                            Some(remote.clone())
                        );
                        
                        if let Some(updated) = dialog.run() {
                            // Update the remote
                            let _ = cm.update_remote(updated);
                        }
                    }
                }
                Err(e) => tracing::error!("Failed to load: {}", e),
            }
        }
    }
});
```

**Key Points:**
- `list.index_of(row)` gets the 0-based index of activated row
- Use this index to access data from your config
- Pre-fill dialog with existing data for editing
- Use `update_remote()` instead of `add_remote()`

### Pattern 3: Combo Box Change Handler

Used when dropdown selection changes:

```rust
combo.connect_changed(move |combo| {
    if let Some(active_text) = combo.active_text() {
        tracing::info!("Selected: {}", active_text);
        // Handle selection change
    }
});
```

### Pattern 4: Entry Field Change Handler

Used when text input changes:

```rust
entry.connect_changed(move |entry| {
    let text = entry.text();
    tracing::info!("Text changed: {}", text);
    // Validate or process text
});
```

## Error Handling Strategy

Always provide user feedback for errors:

```rust
// Bad: Silent failure
match cm.add_remote(config) {
    Ok(_) => {},
    Err(_) => {} // User doesn't know what went wrong
}

// Good: User feedback
match cm.add_remote(config) {
    Ok(_) => {
        tracing::info!("Remote added successfully");
        widgets::show_info_dialog(&window, "Success", "Remote added!");
    }
    Err(e) => {
        tracing::error!("Failed to add remote: {}", e);
        widgets::show_error_dialog(
            &window,
            "Error",
            &format!("Failed to add remote: {}", e),
        );
    }
}
```

## Common Gotchas

### 1. Mutable Lock Panic

```rust
// DON'T: Can panic if lock is poisoned
let mut cm = config_manager.lock().unwrap();

// DO: Handle lock failure gracefully
if let Ok(mut cm) = config_manager.lock() {
    // Use cm
}
```

### 2. Value Already Moved

```rust
// DON'T: Can't use remote_config after move
if let Some(remote_config) = dialog.run() {
    let _ = cm.add_remote(remote_config);
    println!("{}", remote_config.name); // ERROR: value moved
}

// DO: Use references or clone before use
if let Some(remote_config) = dialog.run() {
    let name = remote_config.name.clone();
    let _ = cm.add_remote(remote_config);
    println!("{}", name); // OK
}
```

### 3. Missing Arc Clone

```rust
// DON'T: Compiler error - config_manager is moved
button.connect_clicked(move |_| {
    let _ = config_manager.lock(); // ERROR: config_manager already moved
});
button.connect_clicked(move |_| {
    let _ = config_manager.lock(); // ERROR: can't use twice
});

// DO: Clone Arc before each handler
let cm1 = Arc::clone(&config_manager);
button1.connect_clicked(move |_| {
    let _ = cm1.lock();
});

let cm2 = Arc::clone(&config_manager);
button2.connect_clicked(move |_| {
    let _ = cm2.lock();
});
```

## Workflow Example: Add Remote Button

Here's the complete workflow from button click to config saved:

### 1. User clicks "Add Remote" button
```rust
// Handler is triggered
add_remote_button.connect_clicked(move |_| {
```

### 2. Dialog is created and shown
```rust
let dialog = AddRemoteDialog::new(&window_clone, None);
if let Some(remote_config) = dialog.run() {
```

### 3. User fills in form and clicks Save
- Dialog validates input
- Returns `Some(RemoteConfig)` on successful submit
- Returns `None` on Cancel

### 4. Remote is added to config
```rust
if let Ok(mut cm) = config_manager.lock() {
    match cm.add_remote(remote_config.clone()) {
        Ok(_) => {
            tracing::info!("Remote {} added", remote_config.name);
            // TODO: Refresh list to show new remote
        }
        Err(e) => {
            tracing::error!("Failed to add remote: {}", e);
            show_error_dialog(...);
        }
    }
}
```

### 5. Config file is persisted
- `add_remote()` calls `write_remote()`
- `write_remote()` updates `~/.config/rclone/rclone.conf`

## Testing Event Handlers

### Unit Tests for Business Logic

Test the underlying methods without GTK:

```rust
#[test]
fn test_add_remote() {
    let cm = RcloneConfigManager::new().unwrap();
    let remote = RemoteConfig::new("test", CloudService::GoogleDrive);
    
    assert!(cm.add_remote(remote.clone()).is_ok());
    assert!(cm.add_remote(remote).is_err()); // Duplicate fails
}
```

### Integration Tests with Mock Dialog

Since GTK is not available in test environments, create mock dialogs:

```rust
#[cfg(test)]
struct MockDialog;

impl MockDialog {
    fn run(&self) -> Option<RemoteConfig> {
        Some(RemoteConfig::new("test", CloudService::GoogleDrive))
    }
}
```

### Manual Testing

1. Build the application (requires GTK4 system libraries)
2. Run: `cargo run --bin rclone-config-manager`
3. Click "Add Remote"
4. Fill in form and submit
5. Check `~/.config/rclone/rclone.conf` for changes

## Advanced Patterns

### Async Event Handler

For long-running operations, use `glib::spawn_local()`:

```rust
button.connect_clicked(move |_| {
    let cm = Arc::clone(&config_manager);
    
    glib::spawn_local(async move {
        // This runs in the main thread's event loop
        // but doesn't block the UI
        match async_operation(&cm).await {
            Ok(_) => {},
            Err(e) => tracing::error!("Error: {}", e),
        }
    });
});
```

### Signal Debouncing

For high-frequency signals (like text input), debounce:

```rust
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

let last_change = Rc::new(RefCell::new(Instant::now()));

entry.connect_changed(move |entry| {
    let mut last = last_change.borrow_mut();
    
    if last.elapsed().as_millis() < 500 {
        return; // Ignore rapid changes
    }
    
    *last = Instant::now();
    // Process the change
});
```

## References

- [GTK4 Signals Documentation](https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/prelude/trait.SignalHandlerExt.html)
- [GLib-rs Closures](https://gtk-rs.org/gtk4-rs/stable/latest/docs/glib/source/index.html)
- [Arc and Mutex in The Rust Book](https://doc.rust-lang.org/book/ch16-03-shared-state.html)
- [GTK4 and Rust Best Practices](https://github.com/gtk-rs/gtk4-rs)

## Next Steps

1. Implement Edit/Delete button handlers for rows
2. Add refresh signals after data changes
3. Implement Settings tab save/reset handlers
4. Add mount/unmount toggle handlers
5. Implement OAuth flow in background thread
