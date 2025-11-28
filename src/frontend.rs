/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Frontend (browser) integration for Harbor
//!
//! This module provides a loosely-coupled integration with Servo-based browsers.
//! Instead of linking Servo as a library dependency (which causes version conflicts),
//! Harbor launches the browser as a subprocess. This approach:
//!
//! - Avoids dependency conflicts with Servo's complex dependency tree
//! - Works with any compatible browser binary (servoshell, patched versions, etc.)
//! - Allows upgrading Servo independently of Harbor
//! - Supports fallback to system browsers for development
//!
//! # Browser Discovery
//!
//! Harbor searches for browsers in this order:
//! 1. `HARBOR_BROWSER` environment variable (explicit path)
//! 2. `servoshell` in PATH
//! 3. Fallback options (xdg-open, open, etc.)

use log::{debug, info, warn};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use thiserror::Error;

/// Errors from frontend/browser operations
#[derive(Debug, Error)]
pub enum FrontendError {
    #[error("No suitable browser found. Set HARBOR_BROWSER or install servoshell.")]
    NoBrowserFound,

    #[error("Failed to start browser: {0}")]
    StartFailed(String),

    #[error("Browser exited unexpectedly: {0}")]
    BrowserCrashed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Configuration for launching a browser window
#[derive(Debug, Clone)]
pub struct WindowConfig {
    /// URL to load (can be transport-aware URL)
    pub url: String,
    /// Window title
    pub title: String,
    /// Window width in pixels
    pub width: u32,
    /// Window height in pixels
    pub height: u32,
    /// Whether the window is resizable
    pub resizable: bool,
    /// Whether to show window decorations
    pub decorated: bool,
    /// Whether to start fullscreen
    pub fullscreen: bool,
    /// Whether to enable developer tools
    pub devtools: bool,
}

/// Represents a running browser instance
pub struct BrowserProcess {
    child: Child,
    browser_type: BrowserType,
}

impl BrowserProcess {
    /// Check if the browser is still running
    pub fn is_running(&mut self) -> bool {
        match self.child.try_wait() {
            Ok(None) => true,
            Ok(Some(_)) => false,
            Err(_) => false,
        }
    }

    /// Wait for the browser to exit
    pub fn wait(&mut self) -> Result<i32, FrontendError> {
        let status = self.child.wait()?;
        Ok(status.code().unwrap_or(-1))
    }

    /// Kill the browser process
    pub fn kill(&mut self) -> Result<(), FrontendError> {
        self.child.kill()?;
        Ok(())
    }

    /// Get the browser type
    pub fn browser_type(&self) -> &BrowserType {
        &self.browser_type
    }
}

/// Type of browser being used
#[derive(Debug, Clone, PartialEq)]
pub enum BrowserType {
    /// Servoshell (stock or patched)
    Servoshell(PathBuf),
    /// System browser via xdg-open/open
    SystemBrowser,
    /// Custom browser specified by user
    Custom(PathBuf),
}

/// Finds and launches browsers for Harbor apps
pub struct BrowserLauncher {
    /// Explicitly configured browser path
    browser_path: Option<PathBuf>,
    /// Whether to use system browser as fallback
    allow_system_fallback: bool,
}

impl Default for BrowserLauncher {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserLauncher {
    /// Create a new browser launcher with default settings
    pub fn new() -> Self {
        Self {
            browser_path: None,
            allow_system_fallback: true,
        }
    }

    /// Set an explicit browser path
    pub fn with_browser<P: Into<PathBuf>>(mut self, path: P) -> Self {
        self.browser_path = Some(path.into());
        self
    }

    /// Disable system browser fallback
    pub fn no_fallback(mut self) -> Self {
        self.allow_system_fallback = false;
        self
    }

    /// Find the best available browser
    pub fn find_browser(&self) -> Result<BrowserType, FrontendError> {
        // 1. Check explicit configuration
        if let Some(ref path) = self.browser_path {
            if path.exists() {
                return Ok(BrowserType::Custom(path.clone()));
            }
            warn!("Configured browser not found: {}", path.display());
        }

        // 2. Check HARBOR_BROWSER environment variable
        if let Ok(browser_path) = std::env::var("HARBOR_BROWSER") {
            let path = PathBuf::from(&browser_path);
            if path.exists() {
                info!("Using browser from HARBOR_BROWSER: {}", browser_path);
                return Ok(BrowserType::Custom(path));
            }
            warn!("HARBOR_BROWSER path not found: {}", browser_path);
        }

        // 3. Look for servoshell in PATH
        if let Some(path) = find_in_path("servoshell") {
            info!("Found servoshell in PATH: {}", path.display());
            return Ok(BrowserType::Servoshell(path));
        }

        // 4. Look for common Servo installation locations
        let common_paths = [
            // Development build
            "./target/release/servoshell",
            "./target/debug/servoshell",
            // User-local installation
            "~/.local/bin/servoshell",
            "~/.cargo/bin/servoshell",
            // System installation
            "/usr/local/bin/servoshell",
            "/usr/bin/servoshell",
        ];

        for path_str in common_paths {
            let path = expand_path(path_str);
            if path.exists() {
                info!("Found servoshell at: {}", path.display());
                return Ok(BrowserType::Servoshell(path));
            }
        }

        // 5. System browser fallback
        if self.allow_system_fallback {
            #[cfg(target_os = "linux")]
            if find_in_path("xdg-open").is_some() {
                warn!("No Servo found, falling back to system browser (xdg-open)");
                warn!("Note: Transport URLs (http::unix://) may not work with system browsers");
                return Ok(BrowserType::SystemBrowser);
            }

            #[cfg(target_os = "macos")]
            {
                warn!("No Servo found, falling back to system browser (open)");
                warn!("Note: Transport URLs (http::unix://) may not work with system browsers");
                return Ok(BrowserType::SystemBrowser);
            }
        }

        Err(FrontendError::NoBrowserFound)
    }

    /// Launch a browser with the given window configuration
    pub fn launch(&self, config: &WindowConfig) -> Result<BrowserProcess, FrontendError> {
        let browser_type = self.find_browser()?;

        match browser_type {
            BrowserType::Servoshell(ref path) | BrowserType::Custom(ref path) => {
                self.launch_servoshell(path.clone(), config, browser_type)
            }
            BrowserType::SystemBrowser => {
                self.launch_system_browser(config)
            }
        }
    }

    /// Launch servoshell with appropriate arguments
    fn launch_servoshell(
        &self,
        path: PathBuf,
        config: &WindowConfig,
        browser_type: BrowserType,
    ) -> Result<BrowserProcess, FrontendError> {
        let mut cmd = Command::new(&path);

        // Set window size
        cmd.arg("--window-size");
        cmd.arg(format!("{}x{}", config.width, config.height));

        // Handle headless mode would go here if needed
        // cmd.arg("--headless");

        // URL must be last argument
        cmd.arg(&config.url);

        // Set window title via environment (servoshell may support this)
        cmd.env("HARBOR_WINDOW_TITLE", &config.title);

        // Pass through logging
        if std::env::var("RUST_LOG").is_err() {
            cmd.env("RUST_LOG", "warn");
        }

        debug!("Launching: {:?}", cmd);

        let child = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| FrontendError::StartFailed(format!("{}: {}", path.display(), e)))?;

        info!("Browser started with PID: {}", child.id());

        Ok(BrowserProcess {
            child,
            browser_type,
        })
    }

    /// Launch system browser as fallback
    fn launch_system_browser(&self, config: &WindowConfig) -> Result<BrowserProcess, FrontendError> {
        // Convert transport URL to regular URL for system browsers
        let url = convert_transport_url(&config.url);

        #[cfg(target_os = "linux")]
        let cmd_name = "xdg-open";
        #[cfg(target_os = "macos")]
        let cmd_name = "open";
        #[cfg(target_os = "windows")]
        let cmd_name = "start";
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        let cmd_name = "xdg-open";

        let mut cmd = Command::new(cmd_name);
        cmd.arg(&url);

        debug!("Launching system browser: {} {}", cmd_name, url);

        let child = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| FrontendError::StartFailed(format!("{}: {}", cmd_name, e)))?;

        Ok(BrowserProcess {
            child,
            browser_type: BrowserType::SystemBrowser,
        })
    }
}

/// Find an executable in PATH
fn find_in_path(name: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|p| p.join(name))
            .find(|p| p.exists() && p.is_file())
    })
}

/// Expand ~ in paths
fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

/// Convert transport-aware URL to standard URL
///
/// For system browsers that don't understand transport URLs,
/// we need to convert them. This is a best-effort conversion.
fn convert_transport_url(url: &str) -> String {
    // Transport URL format: scheme::transport//path
    // Example: http::unix///tmp/app.sock/api

    if let Some(rest) = url.strip_prefix("http::unix//") {
        // Unix socket URL - can't be used with regular browsers
        // Try to extract the path portion after the socket
        if let Some(slash_pos) = rest[1..].find('/') {
            // There's a path after the socket
            let path = &rest[slash_pos + 1..];
            warn!("Converting Unix socket URL to localhost (path: {})", path);
            format!("http://localhost/{}", path)
        } else {
            warn!("Unix socket URL can't be converted for system browser");
            "http://localhost/".to_string()
        }
    } else if let Some(rest) = url.strip_prefix("http::tcp//") {
        // Explicit TCP - just convert to regular http://
        format!("http://{}", rest)
    } else if let Some(rest) = url.strip_prefix("https::tcp//") {
        format!("https://{}", rest)
    } else {
        // Already a regular URL
        url.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_transport_url() {
        assert_eq!(
            convert_transport_url("http::tcp//localhost:8080/api"),
            "http://localhost:8080/api"
        );
        assert_eq!(
            convert_transport_url("https://example.com/path"),
            "https://example.com/path"
        );
    }

    #[test]
    fn test_expand_path() {
        let expanded = expand_path("./relative/path");
        assert_eq!(expanded, PathBuf::from("./relative/path"));
    }
}
