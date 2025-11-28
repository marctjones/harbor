# AI Agent Development Guide for Harbor

This document provides instructions for AI coding assistants (Claude Code, Gemini, Cursor, etc.) working on the Harbor local app framework.

## Project Overview

**Harbor** is a local desktop application framework that enables web frontends to connect to backend servers (gunicorn, nginx, Flask, etc.) over Unix Domain Sockets (Linux/macOS) or Named Pipes (Windows).

## Key Purpose

Harbor's **primary purpose** is accessing web applications running in gunicorn or nginx over UDS connections. While other transports are possible, UDS on Linux/macOS and Named Pipes on Windows are the main focus.

## Repository Structure

```
harbor/
├── Cargo.toml           # Package manifest
├── src/
│   ├── lib.rs           # Library exports
│   ├── config.rs        # TOML configuration parsing
│   ├── backend.rs       # Backend process management
│   └── app.rs           # HarborApp main runner
├── README.md
├── DESIGN.md
└── IMPLEMENTATION_PLAN.md
```

## Coding Standards

### Rust Guidelines
- **Edition**: Rust 2021
- **Error Handling**: `thiserror` for library, `anyhow` for binaries
- **Configuration**: `serde` with TOML format
- **Process Management**: Standard library + nix crate for signals

### Code Style
```rust
// Good: Clear configuration structs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    /// Command to run (e.g., "gunicorn")
    pub command: String,

    /// Arguments to pass
    #[serde(default)]
    pub args: Vec<String>,

    /// Socket path
    pub socket: String,
}

// Good: Proper process lifecycle
impl BackendManager {
    pub fn start(&mut self) -> Result<(), BackendError> {
        // Clean up existing socket
        // Build and spawn command
        // Wait for socket ready
    }

    pub fn stop(&mut self) -> Result<(), BackendError> {
        // Graceful shutdown (SIGTERM)
        // Wait or force kill
        // Clean up socket
    }
}
```

### Configuration Format (TOML)
```toml
[app]
name = "My App"
version = "1.0.0"

[backend]
command = "gunicorn"
args = ["--bind", "unix:/tmp/app.sock", "-w", "4", "app:app"]
socket = "/tmp/app.sock"
workdir = "/path/to/app"
startup_timeout = 30
restart_on_crash = true

[frontend]
url = "http::unix///tmp/app.sock/"
width = 1200
height = 800
title = "My Application"

[settings]
devtools = false
log_level = "info"
```

## Key Concepts

### Transport-Aware URLs
Harbor uses Rigging's URL format:
```
http::unix///tmp/app.sock/           # Absolute path (3 slashes)
http::unix//var/run/app.sock/        # Relative path (2 slashes)
http::pipe//myapp/                   # Windows named pipe
```

### Backend Lifecycle
1. Harbor reads `app.toml` configuration
2. Starts backend process (gunicorn, etc.)
3. Waits for socket to be ready
4. Returns configuration for Servo frontend
5. Monitors backend health
6. Restarts on crash if configured
7. Stops backend on application exit

### Socket Readiness Check
```rust
fn wait_for_socket(&self) -> Result<(), BackendError> {
    loop {
        if socket_path.exists() {
            // Try to connect to verify ready
            if UnixStream::connect(socket_path).is_ok() {
                return Ok(());
            }
        }
        // Check process still running
        // Sleep and retry
    }
}
```

## Development Tasks

### Adding a New Backend Type

1. Document backend requirements in README
2. Create example configuration
3. Test socket creation behavior
4. Add specific error handling if needed

### Adding Windows Support

1. Implement named pipe detection in `backend.rs`
2. Update socket path handling for pipe names
3. Modify process management for Windows
4. Add platform-specific configuration options

### Adding a New Configuration Option

1. Add field to appropriate config struct
2. Add `#[serde(default)]` with default function if optional
3. Update README documentation
4. Add validation in config loading if needed

## Common Commands

```bash
# Build library
cargo build

# Run tests
cargo test

# Build example app
cargo build --example hello-flask

# Check configuration parsing
cargo test test_parse_config
```

## Example Backend Configurations

### Gunicorn (Python/WSGI)
```toml
[backend]
command = "gunicorn"
args = ["--bind", "unix:/tmp/app.sock", "-w", "4", "app:app"]
socket = "/tmp/app.sock"
```

### uWSGI (Python)
```toml
[backend]
command = "uwsgi"
args = ["--socket", "/tmp/app.sock", "--wsgi-file", "app.py", "--callable", "app"]
socket = "/tmp/app.sock"
```

### Nginx
```toml
[backend]
command = "nginx"
args = ["-c", "/path/to/nginx.conf", "-g", "daemon off;"]
socket = "/tmp/nginx.sock"
```

### Flask Development Server
```toml
[backend]
command = "flask"
args = ["run", "--host=unix:/tmp/app.sock"]
socket = "/tmp/app.sock"
env = { FLASK_APP = "app.py" }
```

### Node.js
```toml
[backend]
command = "node"
args = ["server.js"]
socket = "/tmp/app.sock"
env = { SOCKET_PATH = "/tmp/app.sock" }
```

## Important Notes

1. **UDS Focus**: Primary use case is Unix Domain Sockets
2. **No Network**: Backend should not listen on TCP ports
3. **Security**: Socket has user-only permissions (0600)
4. **Process Ownership**: Harbor owns the backend lifecycle
5. **Clean Shutdown**: Always clean up socket on exit

## Error Handling

```rust
#[derive(Debug, Error)]
pub enum BackendError {
    #[error("Failed to start backend: {0}")]
    StartFailed(String),

    #[error("Backend exited unexpectedly: {0}")]
    Crashed(String),

    #[error("Socket not ready after {0} seconds")]
    StartupTimeout(u64),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Integration with Rigging

Harbor uses **Rigging** for browser embedding. Rigging provides a stable API that isolates Harbor from Servo's internal APIs.

### Browser Types from Rigging

```rust
// Re-exported from rigging::embed
pub use rigging::embed::{
    BrowserBuilder,      // Builder for browser instances
    BrowserConfig,       // Configuration struct
    BrowserEvent,        // Events during operation
    EmbedError,          // Error type (as BrowserError)
};

// Convenience function
pub fn run_browser(config: BrowserConfig, callback: Option<...>) -> Result<(), BrowserError>;
```

### Harbor's Role

1. **Configuration**: Parse `app.toml` into `HarborConfig`
2. **Backend Management**: Start/stop/monitor backend process
3. **Frontend Launch**: Create `BrowserConfig` and call Rigging's API

### Flow

```
app.toml → HarborConfig → BackendManager.start()
                       → HarborRunConfig → BrowserConfig → Rigging → Servo
```

### Key Point

**DO NOT** import Servo types directly. Always use Rigging's stable API. When Servo is upgraded, only Rigging's `backend.rs` needs changes.

## Related Projects

- [Rigging](https://github.com/marctjones/rigging) - Servo embedding API and transport library
- [Compass](https://github.com/marctjones/compass) - Privacy browser
- [Corsair](https://github.com/marctjones/corsair) - Tor daemon
- [Servo](https://github.com/servo/servo) - Browser engine (via Rigging)
