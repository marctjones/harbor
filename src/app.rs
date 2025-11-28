/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Harbor application runner

use crate::backend::BackendManager;
use crate::config::HarborConfig;
use log::{error, info};
use thiserror::Error;

/// Errors that can occur with Harbor apps
#[derive(Debug, Error)]
pub enum HarborError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Backend error: {0}")]
    Backend(#[from] crate::backend::BackendError),

    #[error("Frontend error: {0}")]
    Frontend(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// A Harbor application instance
pub struct HarborApp {
    config: HarborConfig,
    backend: Option<BackendManager>,
}

impl HarborApp {
    /// Create a new Harbor app from configuration
    pub fn new(config: HarborConfig) -> Self {
        Self {
            config,
            backend: None,
        }
    }

    /// Load a Harbor app from a TOML file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let config = HarborConfig::load(path)?;
        Ok(Self::new(config))
    }

    /// Get the application name
    pub fn name(&self) -> &str {
        &self.config.app.name
    }

    /// Get the frontend URL
    pub fn url(&self) -> &str {
        &self.config.frontend.url
    }

    /// Get the window title
    pub fn window_title(&self) -> &str {
        self.config
            .frontend
            .title
            .as_deref()
            .unwrap_or(&self.config.app.name)
    }

    /// Get window dimensions
    pub fn window_size(&self) -> (u32, u32) {
        (self.config.frontend.width, self.config.frontend.height)
    }

    /// Start the backend server
    pub fn start_backend(&mut self) -> Result<(), HarborError> {
        info!("Starting backend for app: {}", self.config.app.name);

        let mut backend = BackendManager::new(self.config.backend.clone());
        backend.start()?;

        self.backend = Some(backend);
        Ok(())
    }

    /// Stop the backend server
    pub fn stop_backend(&mut self) -> Result<(), HarborError> {
        if let Some(ref mut backend) = self.backend {
            backend.stop()?;
        }
        self.backend = None;
        Ok(())
    }

    /// Check if backend is running and restart if needed
    pub fn check_backend(&mut self) -> Result<(), HarborError> {
        if let Some(ref mut backend) = self.backend {
            backend.check_and_restart()?;
        }
        Ok(())
    }

    /// Get the backend socket path
    pub fn socket_path(&self) -> &str {
        &self.config.backend.socket
    }

    /// Run the Harbor app (starts backend, returns config for frontend)
    ///
    /// This method starts the backend and returns the configuration needed
    /// to create the Servo-based frontend window. The actual window creation
    /// should be done by the binary using Servo.
    pub fn run(&mut self) -> Result<HarborRunConfig, HarborError> {
        // Start backend
        self.start_backend()?;

        info!(
            "Harbor app '{}' ready at {}",
            self.config.app.name, self.config.frontend.url
        );

        Ok(HarborRunConfig {
            url: self.config.frontend.url.clone(),
            title: self.window_title().to_string(),
            width: self.config.frontend.width,
            height: self.config.frontend.height,
            resizable: self.config.frontend.resizable,
            decorated: self.config.frontend.decorated,
            fullscreen: self.config.frontend.fullscreen,
            devtools: self.config.settings.devtools,
        })
    }

    /// Get the configuration
    pub fn config(&self) -> &HarborConfig {
        &self.config
    }
}

impl Drop for HarborApp {
    fn drop(&mut self) {
        if let Err(e) = self.stop_backend() {
            error!("Error stopping backend on drop: {}", e);
        }
    }
}

/// Configuration returned by run() for creating the frontend window
#[derive(Debug, Clone)]
pub struct HarborRunConfig {
    /// URL to load (transport-aware)
    pub url: String,
    /// Window title
    pub title: String,
    /// Window width
    pub width: u32,
    /// Window height
    pub height: u32,
    /// Whether resizable
    pub resizable: bool,
    /// Whether decorated
    pub decorated: bool,
    /// Whether fullscreen
    pub fullscreen: bool,
    /// Whether to enable devtools
    pub devtools: bool,
}
