# Harbor Implementation Plan

## Overview

Harbor is a local desktop application framework - an alternative to Electron.
Harbor connects web frontends to local backends using **Unix Domain Sockets only**.
Harbor has **NO ability to browse the internet over TCP**. All network access is restricted to local Unix sockets.

### External Link Handling

When a user clicks a link to an external website (not the local app), Harbor should:
1. Detect that the URL is external (not a `http::unix:` transport URL)
2. Open the URL in the OS default web browser using:
   - Linux: `xdg-open`
   - macOS: `open`
   - Windows: `start`
3. Not navigate the Harbor window to the external URL

This matches the behavior of native desktop applications.

## Servo Integration Architecture

### The Approach: Fork servoshell, Keep Servo Upstream

We are NOT wrapping Servo as a library. Instead:

1. **Rigging** = Fork of servoshell's core embedding code (stripped of browser chrome)
2. **Servo** = Upstream with minimal patches (pluggable Connector in `net` component)
3. **Harbor** = Uses Rigging + adds backend management + UDS-only connector

```
┌─────────────────────────────────────────────────────────────────┐
│  HARBOR                              COMPASS (future)            │
│  - No browser chrome                 - Full browser chrome       │
│  - Backend management                - Tor integration           │
│  - UdsConnector only                 - TorConnector              │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  RIGGING (forked from servoshell core)                           │
│  - Window management (winit/surfman)                             │
│  - Servo embedding (EmbedderMethods, WindowMethods)              │
│  - Event loop integration                                        │
│  - Pluggable Connector trait                                     │
│  - Transport-aware URL parsing                                   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  SERVO (upstream, minimal patches)                               │
│  - WebRender, Stylo, Layout, Script, html5ever, etc.            │
│  - Patched: net component with Connector hook                    │
└─────────────────────────────────────────────────────────────────┘
```

### Why Fork servoshell?

1. **servoshell already solved the hard embedding problems** - It implements the complex traits Servo requires (`EmbedderMethods`, `WindowMethods`), integrates winit+surfman+WebRender, handles the event loop dance.

2. **We're replacing the shell, not embedding it** - Harbor IS a shell. We swap servoshell's browser UI for our chromeless app framework.

3. **Easier to track upstream** - Servo components stay upstream. We only maintain a small patch to the `net` component. When Servo updates, we update the submodule and rebase.

### What Rigging Provides

Forked from servoshell (kept):
- `headed_window.rs` → Window management
- `webview.rs` → Servo integration
- `app.rs` → Event loop handling
- Compositor integration
- WebRender setup

Stripped from servoshell (removed):
- Toolbar/URL bar
- Tabs
- Bookmarks
- History UI
- Preferences UI
- Download manager

Added by Rigging:
- `Connector` trait for pluggable networking
- Transport-aware URL parsing (`http::unix:`, `http::pipe:`)
- `UdsConnector` (for Harbor)
- `TcpConnector` (for Compass)

### Servo Patches (Minimal)

Only the `net` component is patched:
1. Add `Connector` trait hook in `http_loader.rs`
2. Support transport-aware URLs in request routing
3. Allow external connector injection

Everything else uses upstream Servo.

### Third-Party Libraries

Harbor and Rigging share these with Servo:

| Library | Version | Purpose |
|---------|---------|---------|
| **winit** | 0.30+ | Window creation and event handling |
| **surfman** | 0.9+ | GPU surface management |
| **euclid** | 0.22 | Geometric types |
| **raw-window-handle** | 0.6 | Window handle abstraction |

### Why Not WebKit/WRY/Tauri?

We explicitly chose Servo because:
1. Full control over the rendering and networking pipeline
2. Ability to enforce UDS-only networking at the engine level
3. Support for Servo development and the open web
4. Single Rust binary, no system WebView dependencies

## Phase 1: Core Framework ✓ COMPLETE

### 1.1 Configuration ✓
- [x] TOML configuration parsing
- [x] AppConfig struct
- [x] BackendConfig struct
- [x] FrontendConfig struct
- [x] SettingsConfig struct
- [x] Default values
- [ ] Configuration validation (deferred to Phase 2)
- [ ] Path expansion (~/..) (deferred to Phase 2)

### 1.2 Backend Manager ✓
- [x] Process spawning
- [x] Socket existence waiting
- [x] Socket connectivity check
- [x] Graceful shutdown (SIGTERM)
- [x] Force kill fallback
- [x] Socket cleanup
- [ ] Stdout/stderr capture (deferred to Phase 2)
- [ ] Log forwarding (deferred to Phase 2)

### 1.3 HarborApp ✓
- [x] Load from file
- [x] Start backend
- [x] Stop backend
- [x] Return run config
- [ ] Health check loop (deferred to Phase 2)
- [ ] Event callbacks (deferred to Phase 2)

### 1.4 CLI ✓
- [x] Run application from app.toml
- [x] Example runner (--example flag)
- [x] Backend-only mode (--backend-only flag)
- [x] Verbose logging (-v, -vv, -vvv)

## Phase 2: Robustness

### 2.1 Error Handling
- [x] Basic error types
- [ ] Detailed error messages
- [ ] Error recovery suggestions
- [ ] Structured error logging

### 2.2 Process Management
- [ ] PID file management
- [ ] Orphan process cleanup
- [ ] Signal handling (SIGINT, SIGTERM)
- [ ] Restart throttling

### 2.3 Socket Management
- [ ] Socket permission setting
- [ ] Abstract socket support (Linux)
- [ ] Socket in XDG runtime dir

## Phase 3: Windows Support

### 3.1 Named Pipe Backend
- [ ] Named pipe creation
- [ ] Pipe security descriptor
- [ ] Pipe connectivity check
- [ ] Windows process management

### 3.2 Platform Abstraction
- [ ] Unified socket/pipe interface
- [ ] Platform-specific paths
- [ ] Process signals on Windows

## Phase 4: Advanced Features

### 4.1 Multiple Backends
- [ ] Multiple backend config sections
- [ ] Dependency ordering
- [ ] Combined health checking

### 4.2 Backend Communication
- [ ] Health check endpoint
- [ ] Reload signal
- [ ] Metrics endpoint

### 4.3 Logging
- [ ] Backend log capture
- [ ] Log rotation
- [ ] Log level filtering
- [ ] Structured logging

## Phase 5: Servo Integration (Current Focus) - BLOCKED ON RIGGING

> **⚠️ BLOCKED**: This phase cannot proceed until Rigging completes its servoshell fork.
> See `/home/marc/rigging/IMPLEMENTATION_PLAN.md` for detailed Rigging tasks.
>
> **What Harbor is waiting for from Rigging:**
> 1. Rigging must fork servoshell's core embedding code (~2,500 lines)
> 2. Rigging must strip browser chrome (toolbar, tabs, bookmarks, etc.)
> 3. Rigging must add the pluggable `Connector` trait
> 4. Rigging must implement `UdsConnector` for Harbor
> 5. Rigging must expose the `WebView` public API
>
> **Once Rigging is ready**, Harbor can:
> - Use `rigging::WebView` with `UdsConnector`
> - Handle `WebViewEvent::NavigationRequest` for external links
> - Test with the Flask example over real Unix sockets

**Status**: This phase requires forking servoshell into Rigging, stripping browser chrome, and adding the pluggable Connector trait.

### 5.1 Fork servoshell into Rigging
- [ ] Clone servoshell code into Rigging repo
- [ ] Identify and keep core embedding files:
  - [ ] `headed_window.rs` - window management
  - [ ] `webview.rs` - Servo integration
  - [ ] `app.rs` - event loop handling
  - [ ] Compositor integration code
- [ ] Remove browser chrome:
  - [ ] Toolbar/URL bar
  - [ ] Tab management
  - [ ] Bookmarks
  - [ ] History UI
  - [ ] Preferences UI
  - [ ] Download manager
  - [ ] Minibrowser UI
- [ ] Verify stripped Rigging builds and runs

### 5.2 Add Pluggable Connector Trait
- [ ] Define `Connector` trait in Rigging:
  ```rust
  pub trait Connector: Send + Sync {
      fn allows_url(&self, url: &TransportUrl) -> bool;
      fn connect(&self, url: &TransportUrl) -> Result<...>;
  }
  ```
- [ ] Implement `UdsConnector` (for Harbor)
- [ ] Implement `TcpConnector` (for Compass, standard browsing)
- [ ] Add transport-aware URL parsing (`http::unix:`, `http::pipe:`)

### 5.3 Patch Servo's net Component
- [ ] Add Connector hook to `http_loader.rs`
- [ ] Allow external connector injection at initialization
- [ ] Route requests through injected Connector
- [ ] Test that UdsConnector blocks TCP URLs
- [ ] Test that TcpConnector allows normal browsing

### 5.4 Create Rigging's Public API
- [ ] `WebViewConfig` struct
- [ ] `WebView::new(config, connector, window)` constructor
- [ ] `WebView::navigate(url)` method
- [ ] `WebView::tick()` for event loop
- [ ] `WebView::resize()` for window changes
- [ ] `WebView::handle_input()` for keyboard/mouse
- [ ] `WebView::render()` for drawing
- [ ] `WebViewEvent` enum for callbacks

### 5.5 Integrate Harbor with Rigging
- [ ] Update Harbor to use Rigging's `WebView` API
- [ ] Pass `UdsConnector` to block TCP
- [ ] Handle `WebViewEvent::NavigationRequest` for external links
- [ ] Open external URLs in OS browser (`xdg-open`, `open`, `start`)
- [ ] Test with Flask example over real socket

### 5.6 Window Integration (in Rigging)
- [ ] winit window creation helpers
- [ ] surfman GPU surface setup
- [ ] WebRender rendering context
- [ ] Event loop integration
- [ ] Keyboard/mouse event forwarding
- [ ] Resize handling

### 5.7 Testing
- [ ] Unit tests for Connector trait implementations
- [ ] Unit tests for transport URL parsing
- [ ] Integration test: Rigging renders static HTML
- [ ] Integration test: Harbor + Flask over UDS
- [ ] Integration test: External link opens OS browser
- [ ] Integration test: TCP URLs blocked in Harbor

## Phase 6: Packaging

### 6.1 Application Bundling
- [ ] Linux: AppImage helper
- [ ] macOS: .app bundle helper
- [ ] Windows: MSIX helper

### 6.2 Examples
- [x] Flask example
- [ ] FastAPI example
- [ ] Node.js example
- [ ] Nginx static site example

## Milestones

### v0.1.0 - Basic Framework ✓ COMPLETE
- [x] Configuration parsing
- [x] Single backend management
- [x] Socket-based connectivity
- [x] CLI interface

### v0.2.0 - Servo Integration (Current Target) - NOT STARTED
- [ ] Fork servoshell into Rigging (strip browser chrome)
- [ ] Add pluggable Connector trait to Rigging
- [ ] Patch Servo's net component with Connector hook
- [ ] Implement UdsConnector for Harbor
- [ ] Create Rigging's public WebView API
- [ ] Integrate Harbor with Rigging
- [ ] Window rendering with WebRender via winit/surfman
- [ ] UDS-only networking enforced (block TCP)
- [ ] External link delegation to OS browser
- [ ] Integration test: Harbor + Flask over UDS

### v0.3.0 - Production Ready
- [ ] Robust error handling
- [ ] Log capture
- [ ] Health monitoring

### v0.4.0 - Windows Support
- [ ] Named pipe backend
- [ ] Cross-platform API

### v0.5.0 - Advanced Features
- [ ] Multiple backends
- [ ] Metrics/health endpoints
- [ ] Reload support

### v1.0.0 - Stable Release
- [ ] Full documentation
- [ ] Packaging helpers
- [ ] Comprehensive examples

## Technical Debt

1. **Socket Permissions**: Need explicit mode setting
2. **Log Handling**: Backend logs currently lost
3. **Timeout Config**: Should be per-operation

## Dependencies

### Harbor Dependencies (Current)

| Crate | Version | Purpose |
|-------|---------|---------|
| serde | 1.x | Serialization |
| toml | 0.8.x | Config parsing |
| thiserror | 1.x | Error handling |
| log | 0.4.x | Logging |
| nix | 0.29.x | Unix signals |
| tokio | 1.x | Async runtime |
| clap | 4.x | CLI parsing |
| rigging | path | Servo embedding (forked servoshell) |

### Rigging Dependencies (Phase 5)

Rigging inherits most dependencies from servoshell. Key ones:

| Crate | Version | Purpose |
|-------|---------|---------|
| winit | 0.30+ | Window creation, events |
| surfman | 0.9+ | GPU surface management |
| euclid | 0.22 | Geometric types |
| raw-window-handle | 0.6 | Window handle abstraction |
| servo | submodule | Rendering engine (with net patches) |

### Servo (via Rigging submodule)

Servo brings its own dependencies. We don't manage these directly - they come from the Servo build.

## Example Applications

### Hello Flask
```
examples/hello-flask/
├── app.toml           # Harbor configuration
├── app.py             # Flask application
└── requirements.txt   # Python dependencies
```

### Static Site (Nginx)
```
examples/static-nginx/
├── app.toml           # Harbor configuration
├── nginx.conf         # Nginx configuration
└── public/            # Static files
    └── index.html
```

### Full Stack (FastAPI + React)
```
examples/fullstack/
├── app.toml           # Harbor configuration
├── backend/           # FastAPI backend
│   └── main.py
└── frontend/          # React frontend
    └── build/
```

## Testing Strategy

### Unit Tests
- Configuration parsing
- Default value application
- Path handling

### Integration Tests
- Backend start/stop
- Socket connectivity
- Health checking

### Manual Tests
- Various backend types
- Long-running stability
- Resource cleanup

## Contributing

See AGENTS.md for AI assistant guidelines and coding standards.
