# AI Agent Development Guide for Harbor

This document provides instructions for AI coding assistants (Claude Code, Gemini, Cursor, etc.) working on the Harbor local app framework.

**IMPORTANT**: Read this ENTIRE document before writing any code. Pay special attention to the "Common Mistakes to Avoid" section at the end.

## Project Overview

**Harbor** is a local desktop application framework - an **Electron alternative** built in Rust. It embeds Servo's rendering engine to display web UIs that communicate with local backends over Unix Domain Sockets.

**This is NOT a web browser.** Harbor cannot access the internet. It renders web content from local applications only.

## Key Purpose

Harbor's **primary purpose** is:
1. **Embed Servo's rendering engine** directly into a native Rust binary
2. **Render web UIs** from local backends (gunicorn, Flask, etc.)
3. **Use Unix Domain Sockets exclusively** - NO TCP networking at all
4. **Replace Electron** with a lighter, more secure, Rust-native solution

The architecture is:
```
Your Web App (Flask/etc) ←→ Unix Socket ←→ Harbor (Servo rendering) ←→ User
```

## What This Project IS vs IS NOT

| Harbor IS | Harbor IS NOT |
|-----------|---------------|
| An Electron alternative | A web browser |
| A native Rust app with embedded Servo | A wrapper around system WebView |
| UDS-only networking | Capable of TCP/internet access |
| A single integrated binary | Multiple processes communicating |
| Using Servo for rendering | Using WebKit, Chromium, or Gecko |

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

## Servo Integration Architecture

### The Goal: Embedded Servo Rendering

We are embedding Servo's web rendering capabilities directly into Harbor. This is the hard part of the project. We are NOT using WebKit, Chromium, WRY, Tauri, or any system WebView.

### The Approach: Fork servoshell, Keep Servo as Upstream

**Rigging is a fork of servoshell's core embedding code**, stripped of browser chrome. This is critical to understand:

- **servoshell** = Servo's reference shell (browser UI + embedding code)
- **Rigging** = servoshell's embedding code only (no browser UI) + pluggable networking
- **Harbor** = Rigging + backend management + UDS-only connector
- **Compass** = Rigging + browser UI + Tor connector

```
┌─────────────────────────────────────────────────────────────────┐
│                    APPLICATIONS                                  │
├─────────────────────────────┬───────────────────────────────────┤
│         HARBOR              │           COMPASS                  │
│  - No browser chrome        │  - Full browser chrome             │
│  - Backend management       │  - Tor integration                 │
│  - UDS-only networking      │  - Privacy features                │
│  - app.toml config          │  - .onion support                  │
└─────────────────────────────┴───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    RIGGING                                       │
│            (forked from servoshell core)                         │
├─────────────────────────────────────────────────────────────────┤
│  FROM SERVOSHELL (keep):           STRIPPED (removed):          │
│  - Window management (winit)       - Toolbar/URL bar            │
│  - GPU surface (surfman)           - Tabs                        │
│  - Servo embedding traits          - Bookmarks                   │
│  - Event loop integration          - History UI                  │
│  - WebRender setup                 - Preferences UI              │
│  - Compositor integration          - Download manager            │
├─────────────────────────────────────────────────────────────────┤
│  ADDED BY RIGGING:                                               │
│  - Pluggable Connector trait                                     │
│  - Transport-aware URL parsing (http::unix:, http::pipe:, etc.) │
│  - UdsConnector (for Harbor)                                     │
│  - TcpConnector (for Compass, standard browsing)                 │
│  - TorConnector (for Compass, onion routing)                     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SERVO (upstream)                              │
│              (minimal patches, track upstream)                   │
├─────────────────────────────────────────────────────────────────┤
│  USED AS-IS:                       PATCHED (minimal):           │
│  - WebRender                       - net component:              │
│  - Stylo (CSS)                       - Transport URL support     │
│  - Layout                            - Pluggable connector hook  │
│  - Script (DOM + SpiderMonkey)                                   │
│  - html5ever                                                     │
│  - Fonts                                                         │
│  - Canvas                                                        │
│  - Compositing                                                   │
└─────────────────────────────────────────────────────────────────┘
```

### Why Fork servoshell Instead of Wrapping Servo?

1. **servoshell already solved the hard embedding problems**
   - Implements `EmbedderMethods` and `WindowMethods` traits
   - Integrates winit + surfman + WebRender correctly
   - Handles the complex event loop dance
   - It already works

2. **We're replacing the shell, not embedding it**
   - Harbor IS a shell (like servoshell, but different)
   - We're not putting Servo "inside" something else
   - We're swapping servoshell's browser UI for our app framework

3. **Easier to track upstream Servo**
   - Servo components stay upstream (minimal patches)
   - Rigging rebases on servoshell embedding improvements
   - Harbor/Compass just update their Rigging dependency

### What Each Layer Provides

**Servo (upstream with minimal patches):**
- WebRender, Stylo, Layout, Script, html5ever, Fonts, Canvas, Compositing
- Patched: `net` component with pluggable connector support

**Rigging (forked from servoshell core):**
- Window management (from servoshell's `headed_window.rs`)
- Servo embedding (from servoshell's `webview.rs`)
- Event handling (from servoshell's `app.rs`)
- Compositor integration
- `Connector` trait for pluggable networking
- Transport-aware URL parsing (`http::unix:`, `http::pipe:`, etc.)
- Built-in connectors: `UdsConnector`, `TcpConnector`

**Harbor (uses Rigging):**
- `app.toml` configuration
- Backend process management (gunicorn, Flask, etc.)
- `UdsConnector` - blocks all non-UDS URLs
- External link delegation (open in OS browser)
- No browser chrome

**Compass (uses Rigging):**
- Browser chrome (toolbar, tabs, bookmarks)
- `TorConnector` via Corsair
- Privacy settings
- .onion URL support

### Rigging's Pluggable Connector API

```rust
// The key abstraction that lets Harbor and Compass share Rigging

/// Trait for custom network connectors
pub trait Connector: Send + Sync {
    /// Check if this connector allows the given URL
    fn allows_url(&self, url: &TransportUrl) -> bool;

    /// Connect to the given URL, returning a stream
    fn connect(&self, url: &TransportUrl) -> Result<Box<dyn AsyncReadWrite>, ConnectError>;
}

/// Harbor's connector - UDS only, blocks everything else
pub struct UdsConnector;

impl Connector for UdsConnector {
    fn allows_url(&self, url: &TransportUrl) -> bool {
        matches!(url.transport, Transport::Unix | Transport::Pipe)
    }

    fn connect(&self, url: &TransportUrl) -> Result<Box<dyn AsyncReadWrite>, ConnectError> {
        match &url.transport {
            Transport::Unix(path) => Ok(Box::new(UnixStream::connect(path)?)),
            Transport::Pipe(name) => Ok(Box::new(NamedPipeClient::connect(name)?)),
            _ => Err(ConnectError::Blocked),
        }
    }
}

/// Standard TCP connector (for Compass normal browsing)
pub struct TcpConnector;

/// Tor connector (for Compass .onion sites)
pub struct TorConnector { /* uses Corsair */ }
```

### Rigging's WebView API

```rust
// What Harbor and Compass import from Rigging

pub struct WebViewConfig {
    pub initial_url: String,
    pub width: u32,
    pub height: u32,
    pub device_pixel_ratio: f32,
}

pub struct WebView { /* contains Servo internals */ }

impl WebView {
    /// Create a new WebView with the given connector
    pub fn new<C: Connector>(
        config: WebViewConfig,
        connector: C,
        window: &Window,  // winit window
    ) -> Result<Self, WebViewError>;

    /// Navigate to a URL
    pub fn navigate(&mut self, url: &str) -> Result<(), WebViewError>;

    /// Process events - call from your event loop
    pub fn tick(&mut self) -> Vec<WebViewEvent>;

    /// Handle window resize
    pub fn resize(&mut self, width: u32, height: u32);

    /// Forward input events
    pub fn handle_input(&mut self, event: InputEvent);

    /// Render the current frame
    pub fn render(&mut self);

    /// Shutdown
    pub fn shutdown(self);
}

pub enum WebViewEvent {
    TitleChanged(String),
    UrlChanged(String),
    LoadStarted,
    LoadComplete,
    NavigationRequest { url: String, is_external: bool },
    Error(String),
}
```

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                          HARBOR                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  - app.toml config                                       │    │
│  │  - BackendManager (gunicorn, etc.)                       │    │
│  │  - UdsConnector (blocks TCP)                             │    │
│  │  - External link → OS browser                            │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                      RIGGING                             │    │
│  │  ┌─────────────────────────────────────────────────┐    │    │
│  │  │  WebView::new(config, UdsConnector, &window)    │    │    │
│  │  │  - Window management (winit/surfman)            │    │    │
│  │  │  - Servo embedding                               │    │    │
│  │  │  - Event loop integration                        │    │    │
│  │  └─────────────────────────────────────────────────┘    │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
└──────────────────────────────┼───────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                      SERVO (upstream)                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │   Script     │  │   Layout     │  │     WebRender        │   │
│  │  (DOM + JS)  │  │  (CSS/Box)   │  │   (GPU Compositor)   │   │
│  └──────────────┘  └──────────────┘  └──────────────────────┘   │
│                              │                                   │
│                              ▼                                   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │    net (with Connector hook) → UdsConnector.connect()    │   │
│  └──────────────────────────────────────────────────────────┘   │
└──────────────────────────────┼───────────────────────────────────┘
                               │
                               ▼
                    ┌──────────────────┐
                    │  Unix Socket     │
                    │  /tmp/app.sock   │
                    └──────────────────┘
                               │
                               ▼
                    ┌──────────────────┐
                    │  gunicorn/Flask  │
                    │  (your app)      │
                    └──────────────────┘
```

### Tracking Upstream

```
Servo (upstream)
    ↓ submodule in Rigging, minimal patches to net component
Rigging (fork of servoshell core + Connector abstraction)
    ↓ cargo dependency
Harbor / Compass (our applications)
```

**When Servo updates:**
1. Update Rigging's Servo submodule
2. Rebase minimal net patches
3. Test Rigging
4. Harbor/Compass get updates via `cargo update`

**When servoshell updates:**
1. Cherry-pick relevant embedding improvements into Rigging
2. Ignore browser chrome changes (we don't have that code)

### Key Design Decisions

1. **Rigging is a servoshell fork** - Not a wrapper, an actual fork of the embedding code

2. **Servo stays upstream** - Minimal patches, easy to track releases

3. **Pluggable Connector trait** - Harbor uses UDS, Compass uses TCP/Tor

4. **Harbor owns backend management** - Rigging doesn't know about gunicorn

5. **Compass owns browser chrome** - Rigging doesn't have tabs/toolbar

## Related Projects

- [Rigging](https://github.com/marctjones/rigging) - Servo embedding API and transport library
- [Compass](https://github.com/marctjones/compass) - Privacy browser (uses Rigging)
- [Corsair](https://github.com/marctjones/corsair) - Tor daemon
- [Servo](https://github.com/servo/servo) - Browser engine (embedded via Rigging)

## Development Workflow: TDD and Commits

### Test-Driven Development

**Write tests BEFORE or ALONGSIDE implementation code.** Do not defer testing.

1. **Unit Tests** - Test individual functions and structs in isolation
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_parse_socket_path() {
           let config = BackendConfig::from_str("...");
           assert_eq!(config.socket, "/tmp/app.sock");
       }
   }
   ```

2. **Integration Tests** - Test components working together (in `tests/` directory)
   ```rust
   // tests/backend_integration.rs
   #[test]
   fn test_backend_starts_and_socket_ready() {
       let mut manager = BackendManager::new(test_config());
       manager.start().unwrap();
       assert!(Path::new("/tmp/test.sock").exists());
       manager.stop().unwrap();
   }
   ```

3. **Test Coverage Goals**
   - All public API functions must have tests
   - Error paths must be tested, not just happy paths
   - Configuration parsing edge cases
   - Socket lifecycle (create, connect, cleanup)

### Commit Frequently

**Commit after every successful test run.** Small, frequent commits are better than large batches.

1. **When to Commit**
   - After adding a new test that passes
   - After implementing a feature that passes its tests
   - After fixing a bug (with regression test)
   - After refactoring (tests still pass)

2. **Commit Message Format**
   ```
   <type>: <short description>

   <optional longer description>
   ```
   Types: `feat`, `fix`, `test`, `refactor`, `docs`, `chore`

3. **Run Tests Before Every Commit**
   ```bash
   cargo test && git add -A && git commit -m "feat: add socket timeout config"
   ```

4. **DO NOT**
   - Commit code that doesn't compile
   - Commit code with failing tests
   - Make huge commits with multiple unrelated changes
   - Wait until "everything is done" to commit

### Test Commands

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_backend_start

# Run tests in a specific module
cargo test config::tests

# Run integration tests only
cargo test --test '*'
```

## Common Mistakes to Avoid

**READ THIS SECTION CAREFULLY.** These are mistakes AI assistants keep making repeatedly:

### 1. DO NOT Suggest Alternative Approaches to Embedding Servo

**WRONG:**
- "Consider using WRY/Tauri instead"
- "You could use the system WebView"
- "WebKit might be easier"
- "Have you considered using webview crate?"

**WHY IT'S WRONG:** The entire point of this project is to embed Servo directly. We chose Servo deliberately. Do not suggest alternatives.

### 2. DO NOT Add TCP Networking

**WRONG:**
- Adding `hyper` with TCP support
- Suggesting `http://localhost:8000` URLs
- Any code that opens TCP sockets

**RIGHT:**
- Unix Domain Sockets only: `http::unix:///tmp/app.sock/`
- Named Pipes on Windows: `http::pipe//pipename/`

### 3. DO NOT Import Servo Types Directly

**WRONG:**
```rust
use servo::compositing::CompositorMsg;
use servo::script::dom::window::Window;
```

**RIGHT:**
```rust
use rigging::embed::{Browser, BrowserConfig, BrowserEvent};
```

**WHY:** Rigging provides a stable API. Servo's internals change frequently.

### 4. DO NOT Treat This as a Web Browser

**WRONG:**
- Adding URL bar functionality
- Implementing bookmarks
- Adding tab support
- Building history management
- Creating a downloads UI

**RIGHT:** This is a desktop app framework. The web UI comes from the local backend.

### 5. DO NOT Skip Reading IMPLEMENTATION_PLAN.md

Before making changes, read `IMPLEMENTATION_PLAN.md` to understand:
- What's already implemented
- What's in progress
- The phased approach we're taking

### 6. DO NOT Propose Half-Measures for Servo Integration

**WRONG:**
- "Let's use a WebView for now and add Servo later"
- "We can stub out the rendering and focus on the backend"
- "Maybe start with a simpler rendering approach"

**RIGHT:** We are implementing Servo embedding. That's the core challenge. Don't avoid it.

### 7. DO NOT Forget the Transport URL Format

**WRONG:**
```
http://localhost/path
unix:///tmp/app.sock
file:///tmp/app.sock
```

**RIGHT:**
```
http::unix///tmp/app.sock/path    # Absolute socket path (3 slashes)
http::unix//relative.sock/path    # Relative socket path (2 slashes)
http::pipe//pipename/path         # Windows named pipe
```

### 8. DO NOT Create Browser Chrome

Harbor is NOT a browser. Do not implement:
- Navigation buttons (back/forward)
- URL input field
- Tab bar
- Bookmarks toolbar
- Status bar with page load progress

The window shows ONLY the web app content, like Electron does.

### 9. DO NOT Ignore External Links

When the user clicks a link to an external URL (not `http::unix:`):
- DO NOT navigate to it (we can't - no TCP)
- DO NOT show an error
- DO open it in the OS default browser (`xdg-open`, `open`, `start`)

### 10. DO NOT Overcomplicate the Backend Manager

The backend manager just:
1. Starts a process (gunicorn, etc.)
2. Waits for the socket to be ready
3. Restarts if it crashes
4. Stops it on exit

Don't add: service discovery, load balancing, multiple backends (yet), health check endpoints, metrics collection.

### 11. DO NOT Skip Writing Tests

**WRONG:**
- "I'll add tests later"
- "Here's the implementation, tests can come in a follow-up"
- Writing 500 lines of code with zero tests

**RIGHT:**
- Write the test first or alongside the implementation
- Every new function gets a test
- Test error cases, not just happy paths
- Run `cargo test` before considering any task complete

### 12. DO NOT Make Giant Uncommitted Changes

**WRONG:**
- Implementing an entire feature across 10 files before committing
- "I'll commit when it's all working"
- Losing work because of uncommitted changes

**RIGHT:**
- Commit after each small, working increment
- Tests pass → commit
- One logical change per commit
- Commit messages explain WHY, not just WHAT
