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
pub mod frontend;

pub use config::HarborConfig;
pub use app::HarborApp;
pub use frontend::{BrowserLauncher, BrowserProcess, BrowserType, WindowConfig};
