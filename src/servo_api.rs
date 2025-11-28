/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Stable API layer for Servo integration
//!
//! This module provides a stable, simplified API for embedding Servo in Harbor.
//! It isolates Harbor's business logic from Servo's internal APIs, making it
//! easier to upgrade Servo versions without rewriting Harbor.
//!
//! # Design Principles
//!
//! 1. **Stability**: This API should remain stable across Servo upgrades
//! 2. **Simplicity**: Expose only what Harbor needs, hide complexity
//! 3. **Isolation**: All Servo-specific types stay within this module
//! 4. **Documentation**: Every change to this API should be documented
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Harbor Application                        │
//! │  - app.rs (HarborApp)                                       │
//! │  - backend.rs (BackendManager)                              │
//! │  - config.rs (HarborConfig)                                 │
//! └────────────────────────┬────────────────────────────────────┘
//!                          │ Uses stable API
//!                          ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                   servo_api.rs (THIS FILE)                   │
//! │  STABLE API BOUNDARY - changes here require version bump    │
//! │  - HarborBrowser                                            │
//! │  - BrowserConfig                                            │
//! │  - run_browser()                                            │
//! └────────────────────────┬────────────────────────────────────┘
//!                          │ Internal implementation
//!                          ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    Servo (with Rigging)                      │
//! │  - ServoBuilder                                             │
//! │  - EventLoop / ApplicationHandler                           │
//! │  - PlatformWindow                                           │
//! │  - Transport URL support (Unix sockets, etc.)               │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Upgrading Servo
//!
//! When upgrading Servo:
//! 1. Update the servo git dependency in Cargo.toml
//! 2. Fix any compilation errors in THIS FILE ONLY
//! 3. Do NOT change the public API unless absolutely necessary
//! 4. If API changes are needed, document them in CHANGELOG.md
//!
//! # API Stability Contract
//!
//! The following are considered stable and should not change:
//! - `BrowserConfig` struct fields
//! - `run_browser()` function signature
//! - `BrowserEvent` enum variants
//!
//! The following may change between Servo versions:
//! - Internal implementation details
//! - Private helper functions
//! - Debug/Display implementations

use log::{debug, error, info, warn};
use std::path::PathBuf;
use thiserror::Error;

// ============================================================================
// STABLE PUBLIC API - Changes here require semver version bump
// ============================================================================

/// Errors from the browser/Servo integration
#[derive(Debug, Error)]
pub enum BrowserError {
    #[error("Failed to initialize browser: {0}")]
    InitFailed(String),

    #[error("Failed to load URL: {0}")]
    LoadFailed(String),

    #[error("Window creation failed: {0}")]
    WindowFailed(String),

    #[error("Event loop error: {0}")]
    EventLoopError(String),

    #[error("Transport URL parsing failed: {0}")]
    TransportUrlError(String),
}

/// Configuration for the browser window
///
/// This struct is part of the stable API. Fields should not be removed,
/// only added with appropriate defaults.
#[derive(Debug, Clone)]
pub struct BrowserConfig {
    /// URL to load (supports transport-aware URLs like `http::unix:///path/`)
    pub url: String,

    /// Window title
    pub title: String,

    /// Window width in pixels
    pub width: u32,

    /// Window height in pixels
    pub height: u32,

    /// Whether the window can be resized
    pub resizable: bool,

    /// Whether to show window decorations (title bar, borders)
    pub decorated: bool,

    /// Whether to start in fullscreen mode
    pub fullscreen: bool,

    /// Whether to enable developer tools (F12)
    pub devtools: bool,

    /// Custom user agent string (None = default)
    pub user_agent: Option<String>,

    /// Path to userscripts directory (optional)
    pub userscripts_dir: Option<PathBuf>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            url: "about:blank".to_string(),
            title: "Harbor".to_string(),
            width: 1024,
            height: 768,
            resizable: true,
            decorated: true,
            fullscreen: false,
            devtools: false,
            user_agent: None,
            userscripts_dir: None,
        }
    }
}

impl BrowserConfig {
    /// Create a new browser config with the given URL
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// Set window title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set window size
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Enable/disable resizing
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    /// Enable/disable window decorations
    pub fn with_decorated(mut self, decorated: bool) -> Self {
        self.decorated = decorated;
        self
    }

    /// Enable/disable fullscreen
    pub fn with_fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    /// Enable/disable devtools
    pub fn with_devtools(mut self, devtools: bool) -> Self {
        self.devtools = devtools;
        self
    }
}

/// Events that can occur during browser operation
///
/// This enum is part of the stable API.
#[derive(Debug, Clone)]
pub enum BrowserEvent {
    /// Browser window was created
    WindowCreated,

    /// Page finished loading
    LoadComplete,

    /// Page title changed
    TitleChanged(String),

    /// User requested to close the window
    CloseRequested,

    /// Browser encountered an error
    Error(String),
}

/// Callback for browser events
pub type BrowserEventCallback = Box<dyn Fn(BrowserEvent) + Send + 'static>;

/// Run the browser with the given configuration
///
/// This is the main entry point for the stable API. It creates a browser
/// window, loads the specified URL, and runs the event loop until the
/// window is closed.
///
/// # Arguments
///
/// * `config` - Browser configuration
/// * `event_callback` - Optional callback for browser events
///
/// # Returns
///
/// Returns `Ok(())` when the browser window is closed normally, or
/// `Err(BrowserError)` if an error occurs.
///
/// # Example
///
/// ```ignore
/// use harbor::servo_api::{BrowserConfig, run_browser};
///
/// let config = BrowserConfig::new("http::unix///tmp/app.sock/")
///     .with_title("My App")
///     .with_size(1200, 800);
///
/// run_browser(config, None)?;
/// ```
pub fn run_browser(
    config: BrowserConfig,
    event_callback: Option<BrowserEventCallback>,
) -> Result<(), BrowserError> {
    info!("Starting browser with URL: {}", config.url);
    debug!("Browser config: {:?}", config);

    // Validate transport URL format
    validate_transport_url(&config.url)?;

    // Run the actual Servo implementation
    run_servo_impl(config, event_callback)
}

/// Check if Servo/browser support is available
///
/// Returns true if the browser can be started. This may return false
/// if required dependencies are missing or Servo wasn't compiled in.
pub fn is_browser_available() -> bool {
    // For now, always return true when Servo is compiled in
    // In the future, this could check for missing dependencies
    cfg!(feature = "servo")
}

/// Get the Servo version string
pub fn servo_version() -> &'static str {
    // This will come from Servo's VERSION constant when integrated
    "0.0.1-dev"
}

// ============================================================================
// INTERNAL IMPLEMENTATION - May change between Servo versions
// ============================================================================

/// Validate a transport-aware URL
fn validate_transport_url(url: &str) -> Result<(), BrowserError> {
    // Basic validation - full parsing happens in Servo's TransportUrl
    if url.is_empty() {
        return Err(BrowserError::TransportUrlError("URL cannot be empty".into()));
    }

    // Check for supported transport schemes
    let supported_prefixes = [
        "http://",
        "https://",
        "http::tcp//",
        "https::tcp//",
        "http::unix//",
        "http::unix///",
        "http::tor//",
        "about:",
        "file://",
    ];

    let is_valid = supported_prefixes.iter().any(|prefix| url.starts_with(prefix));

    if !is_valid {
        warn!("URL may not be a supported transport format: {}", url);
        // Don't error - let Servo try to parse it
    }

    Ok(())
}

/// Internal Servo implementation
///
/// This function contains all the Servo-specific code. When upgrading
/// Servo, changes should be isolated to this function.
///
/// TODO: Full Servo integration requires:
/// 1. Adding servo as a git dependency in Cargo.toml
/// 2. Applying Rigging patches to Servo for transport URL support
/// 3. Implementing the full event loop with winit
///
/// For now, this is a placeholder that demonstrates the API structure.
/// The actual implementation will be added when Servo is integrated.
fn run_servo_impl(
    config: BrowserConfig,
    event_callback: Option<BrowserEventCallback>,
) -> Result<(), BrowserError> {
    // TODO: Full Servo implementation
    //
    // When Servo is added as a dependency, this will:
    // 1. Create a winit EventLoop
    // 2. Build Servo with ServoBuilder
    // 3. Create a window with the specified size
    // 4. Load the URL (with transport support via Rigging patches)
    // 5. Run the event loop until the window is closed
    //
    // Example structure (not yet compiled):
    // ```
    // use servo::config::opts::Opts;
    // use servo::config::prefs::Preferences;
    // use servo::servo_url::ServoUrl;
    // use servo::ServoBuilder;
    // use winit::event_loop::EventLoop;
    //
    // let event_loop = EventLoop::new()?;
    // let servo = ServoBuilder::default()
    //     .opts(Opts::default())
    //     .preferences(Preferences::default())
    //     .build();
    // // ... window creation and event handling
    // ```

    info!("Browser window requested for URL: {}", config.url);
    info!("Window: {}x{}, title: {}", config.width, config.height, config.title);

    // Fire events if callback provided
    if let Some(ref callback) = event_callback {
        callback(BrowserEvent::WindowCreated);
    }

    // For now, show a message about pending Servo integration
    warn!("Servo integration pending - browser window not yet implemented");
    warn!("The backend is running. Use curl to test:");
    warn!("  curl --unix-socket /tmp/your-app.sock http://localhost/");

    // Return error indicating Servo needs to be integrated
    Err(BrowserError::InitFailed(
        "Servo browser engine integration is pending. \
         Use --backend-only flag to run without browser window, \
         or wait for Servo integration to be completed.".into()
    ))
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_config_builder() {
        let config = BrowserConfig::new("http://localhost/")
            .with_title("Test")
            .with_size(800, 600)
            .with_resizable(false);

        assert_eq!(config.url, "http://localhost/");
        assert_eq!(config.title, "Test");
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert!(!config.resizable);
    }

    #[test]
    fn test_validate_transport_url() {
        // Valid URLs
        assert!(validate_transport_url("http://localhost/").is_ok());
        assert!(validate_transport_url("https://example.com/").is_ok());
        assert!(validate_transport_url("http::unix///tmp/app.sock/").is_ok());
        assert!(validate_transport_url("http::tcp//localhost:8080/").is_ok());
        assert!(validate_transport_url("about:blank").is_ok());

        // Empty URL is invalid
        assert!(validate_transport_url("").is_err());
    }

    #[test]
    fn test_browser_config_defaults() {
        let config = BrowserConfig::default();
        assert_eq!(config.url, "about:blank");
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
        assert!(config.resizable);
        assert!(config.decorated);
        assert!(!config.fullscreen);
        assert!(!config.devtools);
    }
}
