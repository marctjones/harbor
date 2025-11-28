/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Harbor - Local Desktop App Framework
//!
//! Harbor enables building desktop applications with web frontends that connect
//! to local backend servers (gunicorn, nginx, Flask, etc.) over Unix Domain
//! Sockets (Linux/macOS) or Named Pipes (Windows).
//!
//! # Overview
//!
//! Harbor provides:
//! - Configuration-based app definition (TOML)
//! - Automatic backend server lifecycle management
//! - Servo-powered web view for the frontend
//! - Transport abstraction via Rigging library
//!
//! # Example
//!
//! ```toml
//! # app.toml
//! [app]
//! name = "My App"
//! version = "1.0.0"
//!
//! [backend]
//! command = "gunicorn"
//! args = ["--bind", "unix:/tmp/myapp.sock", "app:create_app()"]
//! socket = "/tmp/myapp.sock"
//!
//! [frontend]
//! url = "http::unix///tmp/myapp.sock/"
//! width = 1200
//! height = 800
//! ```
//!
//! # Platform Support
//!
//! - **Linux/macOS**: Unix Domain Sockets
//! - **Windows**: Named Pipes (planned)

pub mod config;
pub mod backend;
pub mod app;

pub use config::HarborConfig;
pub use app::HarborApp;

// Re-export browser types from Rigging's stable embedding API
// This provides a consistent interface and isolates Harbor from Servo internals
pub use rigging::embed::{
    BrowserBuilder,
    BrowserConfig,
    BrowserEvent,
    EmbedError as BrowserError,
};

/// Run the browser with the given configuration
///
/// This is a convenience wrapper around Rigging's BrowserBuilder.
pub fn run_browser(
    config: BrowserConfig,
    event_callback: Option<Box<dyn Fn(BrowserEvent) + Send + 'static>>,
) -> Result<(), BrowserError> {
    let mut builder = BrowserBuilder::new().config(config);

    if let Some(callback) = event_callback {
        builder = builder.on_event(callback);
    }

    builder.run()
}

/// Check if browser support is available
///
/// Returns true if Servo browser engine is available.
pub fn is_browser_available() -> bool {
    // For now, check if the servo feature is enabled
    // When Rigging properly integrates Servo, this will delegate to Rigging
    cfg!(feature = "servo")
}
