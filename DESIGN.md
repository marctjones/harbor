# Harbor Design Document

## Overview

Harbor is a local desktop application framework that enables building desktop applications with web frontends connected to local backend servers over Unix Domain Sockets or Named Pipes.

## Problem Statement

Modern desktop applications often want to:
1. Use web technologies (HTML/CSS/JS) for UI
2. Run backend logic in Python, Node.js, or other languages
3. Avoid network exposure for security
4. Have better performance than TCP loopback

Harbor solves this by:
- Managing backend process lifecycle
- Connecting Servo frontend to backend via UDS
- Providing simple TOML-based configuration

## Goals

1. **Simple Configuration**: Single TOML file defines entire app
2. **Backend Agnostic**: Support any HTTP server (gunicorn, nginx, etc.)
3. **Secure by Default**: No network ports, local IPC only
4. **Cross-Platform**: Linux/macOS via UDS, Windows via Named Pipes
5. **Reliable**: Backend health monitoring and auto-restart

## Non-Goals

1. Not a web framework (use Flask, FastAPI, etc.)
2. Not a bundling/packaging tool
3. Not handling backend dependencies

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Harbor Application                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────┐         ┌──────────────────────────┐  │
│  │  Configuration   │         │    Backend Manager       │  │
│  │                  │         │                          │  │
│  │  - app.toml      │────────►│  - Process spawn        │  │
│  │  - Validation    │         │  - Socket monitoring    │  │
│  │  - Defaults      │         │  - Health checks        │  │
│  └──────────────────┘         │  - Auto-restart         │  │
│                               └──────────────────────────┘  │
│                                          │                   │
│  ┌──────────────────┐                    │ UDS              │
│  │  Servo Frontend  │◄───────────────────┘                  │
│  │                  │                                       │
│  │  - Web rendering │         ┌──────────────────────────┐  │
│  │  - User input    │         │   Backend Process        │  │
│  │  - Window mgmt   │◄───────►│                          │  │
│  └──────────────────┘   UDS   │  - gunicorn/nginx/etc    │  │
│                               │  - Application logic     │  │
│                               └──────────────────────────┘  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Configuration Schema

### app.toml Structure

```toml
[app]
name = "Application Name"           # Required
version = "1.0.0"                   # Optional, default: "0.1.0"
icon = "/path/to/icon.png"          # Optional
description = "App description"     # Optional

[backend]
command = "gunicorn"                # Required: executable name
args = ["--bind", "unix:..."]       # Optional: command arguments
socket = "/tmp/app.sock"            # Required: socket path
workdir = "/app/directory"          # Optional: working directory
env = { KEY = "value" }             # Optional: environment vars
startup_timeout = 30                # Optional: seconds, default: 30
restart_on_crash = true             # Optional: default: true

[frontend]
url = "http::unix///tmp/app.sock/"  # Required: transport-aware URL
width = 1200                        # Optional: default: 1024
height = 800                        # Optional: default: 768
title = "Window Title"              # Optional: defaults to app.name
resizable = true                    # Optional: default: true
decorated = true                    # Optional: default: true
fullscreen = false                  # Optional: default: false
min_size = [800, 600]              # Optional: minimum window size
max_size = [1920, 1080]            # Optional: maximum window size

[settings]
devtools = false                    # Optional: default: false
log_level = "info"                  # Optional: default: "info"
user_agent = "Custom UA"            # Optional: custom user agent
```

## Component Design

### HarborConfig

Parses and validates TOML configuration:

```rust
pub struct HarborConfig {
    pub app: AppConfig,
    pub backend: BackendConfig,
    pub frontend: FrontendConfig,
    pub settings: SettingsConfig,
}

impl HarborConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self>;
    pub fn from_str(toml: &str) -> Result<Self>;
}
```

### BackendManager

Manages backend process lifecycle:

```rust
pub struct BackendManager {
    config: BackendConfig,
    process: Option<Child>,
}

impl BackendManager {
    pub fn new(config: BackendConfig) -> Self;
    pub fn start(&mut self) -> Result<()>;
    pub fn stop(&mut self) -> Result<()>;
    pub fn is_running(&mut self) -> bool;
    pub fn check_and_restart(&mut self) -> Result<bool>;
}
```

### HarborApp

Main application runner:

```rust
pub struct HarborApp {
    config: HarborConfig,
    backend: Option<BackendManager>,
}

impl HarborApp {
    pub fn new(config: HarborConfig) -> Self;
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self>;
    pub fn run(&mut self) -> Result<HarborRunConfig>;
}
```

## Socket Path Conventions

### Unix Domain Sockets (Linux/macOS)

```
/tmp/appname.sock           # Temporary (common)
/run/user/1000/app.sock     # XDG runtime directory
~/.local/run/app.sock       # User-specific
```

### Named Pipes (Windows)

```
\\.\pipe\appname            # Standard named pipe path
```

## Backend Process Management

### Startup Sequence

1. Remove existing socket file if present
2. Build command with arguments
3. Set working directory and environment
4. Spawn process with captured stdout/stderr
5. Poll for socket existence
6. Attempt connection to verify readiness
7. Return success or timeout error

### Shutdown Sequence

1. Send SIGTERM to process
2. Wait up to 2 seconds for graceful exit
3. Send SIGKILL if still running
4. Wait for process to exit
5. Remove socket file

### Health Checking

```rust
pub fn check_and_restart(&mut self) -> Result<bool> {
    if !self.is_running() && self.config.restart_on_crash {
        self.process = None;
        self.start()?;
        return Ok(true); // Restarted
    }
    Ok(false) // No restart needed
}
```

## Security Model

### No Network Exposure

- Backend listens only on Unix socket
- No TCP/UDP ports opened
- Not accessible from network

### Socket Permissions

- Created with mode 0600 (user only)
- Only owning user can connect
- No group or world access

### Process Isolation

- Backend runs as user's process
- No elevated privileges needed
- Standard OS process isolation

## Error Handling

```rust
#[derive(Debug, Error)]
pub enum HarborError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Backend error: {0}")]
    Backend(#[from] BackendError),

    #[error("Frontend error: {0}")]
    Frontend(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

## Platform Considerations

### Linux
- Full Unix socket support
- systemd socket activation (future)
- AppImage/Flatpak packaging

### macOS
- Unix socket support
- Sandbox considerations
- .app bundle packaging

### Windows
- Named pipe implementation
- Service integration
- MSIX packaging

## Future Extensions

1. **Multiple Backends**: Support multiple backend processes
2. **Socket Activation**: Systemd socket activation
3. **Hot Reload**: Backend reload without restart
4. **Resource Limits**: CPU/memory limits for backend
5. **Logging**: Backend log capture and routing
