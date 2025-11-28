/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Backend server process management

use crate::config::BackendConfig;
use log::{debug, error, info, warn};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use thiserror::Error;

/// Errors that can occur with backend management
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

/// Manages the backend server process
pub struct BackendManager {
    config: BackendConfig,
    process: Option<Child>,
}

impl BackendManager {
    /// Create a new backend manager
    pub fn new(config: BackendConfig) -> Self {
        Self {
            config,
            process: None,
        }
    }

    /// Start the backend server
    pub fn start(&mut self) -> Result<(), BackendError> {
        info!("Starting backend: {} {:?}", self.config.command, self.config.args);

        // Clean up existing socket file if present
        let socket_path = Path::new(&self.config.socket);
        if socket_path.exists() {
            debug!("Removing existing socket: {}", self.config.socket);
            std::fs::remove_file(socket_path)?;
        }

        // Build command
        let mut cmd = Command::new(&self.config.command);
        cmd.args(&self.config.args);

        // Set working directory
        if let Some(ref workdir) = self.config.workdir {
            cmd.current_dir(workdir);
        }

        // Set environment variables
        for (key, value) in &self.config.env {
            cmd.env(key, value);
        }

        // Capture output for logging
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Spawn process
        let child = cmd.spawn().map_err(|e| {
            BackendError::StartFailed(format!("Failed to spawn {}: {}", self.config.command, e))
        })?;

        self.process = Some(child);
        info!("Backend process started");

        // Wait for socket to be ready
        self.wait_for_socket()?;

        Ok(())
    }

    /// Wait for the backend socket to be ready
    fn wait_for_socket(&mut self) -> Result<(), BackendError> {
        let socket_path = Path::new(&self.config.socket);
        let start = Instant::now();
        let timeout = Duration::from_secs(self.config.startup_timeout);

        info!("Waiting for socket: {}", self.config.socket);

        while start.elapsed() < timeout {
            if socket_path.exists() {
                // Try to connect to verify it's ready
                #[cfg(unix)]
                {
                    use std::os::unix::net::UnixStream;
                    if UnixStream::connect(socket_path).is_ok() {
                        info!("Socket ready: {}", self.config.socket);
                        return Ok(());
                    }
                }

                // Socket file exists, might be ready
                debug!("Socket file exists, checking connectivity...");
            }

            // Check if process is still running
            if let Some(ref mut child) = self.process {
                if let Ok(Some(status)) = child.try_wait() {
                    return Err(BackendError::Crashed(format!(
                        "Backend exited with status: {}",
                        status
                    )));
                }
            }

            std::thread::sleep(Duration::from_millis(100));
        }

        Err(BackendError::StartupTimeout(self.config.startup_timeout))
    }

    /// Stop the backend server
    pub fn stop(&mut self) -> Result<(), BackendError> {
        if let Some(ref mut child) = self.process {
            info!("Stopping backend process");

            // Try graceful shutdown first
            #[cfg(unix)]
            {
                use nix::sys::signal::{kill, Signal};
                use nix::unistd::Pid;

                if let Ok(pid) = child.id().try_into() {
                    let _ = kill(Pid::from_raw(pid), Signal::SIGTERM);

                    // Wait a bit for graceful shutdown
                    std::thread::sleep(Duration::from_secs(2));
                }
            }

            // Force kill if still running
            if child.try_wait()?.is_none() {
                warn!("Backend didn't stop gracefully, forcing kill");
                child.kill()?;
            }

            child.wait()?;
            info!("Backend process stopped");
        }

        self.process = None;

        // Clean up socket file
        let socket_path = Path::new(&self.config.socket);
        if socket_path.exists() {
            let _ = std::fs::remove_file(socket_path);
        }

        Ok(())
    }

    /// Check if the backend is running
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.process {
            match child.try_wait() {
                Ok(None) => true, // Still running
                Ok(Some(_)) => false, // Exited
                Err(_) => false,
            }
        } else {
            false
        }
    }

    /// Restart the backend if it crashed
    pub fn check_and_restart(&mut self) -> Result<bool, BackendError> {
        if !self.is_running() && self.config.restart_on_crash {
            warn!("Backend crashed, restarting...");
            self.process = None;
            self.start()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get the socket path
    pub fn socket_path(&self) -> &str {
        &self.config.socket
    }
}

impl Drop for BackendManager {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            error!("Error stopping backend: {}", e);
        }
    }
}
