/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Harbor application configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main Harbor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarborConfig {
    /// Application metadata
    pub app: AppConfig,

    /// Backend server configuration
    pub backend: BackendConfig,

    /// Frontend window configuration
    pub frontend: FrontendConfig,

    /// Optional: Additional settings
    #[serde(default)]
    pub settings: SettingsConfig,
}

impl HarborConfig {
    /// Load configuration from a TOML file
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: HarborConfig = toml::from_str(&contents)?;
        Ok(config)
    }

    /// Load configuration from string
    pub fn from_str(toml_str: &str) -> anyhow::Result<Self> {
        let config: HarborConfig = toml::from_str(toml_str)?;
        Ok(config)
    }
}

/// Application metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application name
    pub name: String,

    /// Application version
    #[serde(default = "default_version")]
    pub version: String,

    /// Application icon path (optional)
    pub icon: Option<PathBuf>,

    /// Application description (optional)
    pub description: Option<String>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

/// Backend server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    /// Command to run the backend (e.g., "gunicorn", "nginx", "python")
    pub command: String,

    /// Arguments to pass to the command
    #[serde(default)]
    pub args: Vec<String>,

    /// Socket path (Unix) or pipe name (Windows)
    pub socket: String,

    /// Working directory for the backend process
    pub workdir: Option<PathBuf>,

    /// Environment variables to set
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,

    /// Startup timeout in seconds
    #[serde(default = "default_startup_timeout")]
    pub startup_timeout: u64,

    /// Whether to restart on crash
    #[serde(default = "default_restart")]
    pub restart_on_crash: bool,
}

fn default_startup_timeout() -> u64 {
    30
}

fn default_restart() -> bool {
    true
}

/// Frontend window configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendConfig {
    /// URL to load (transport-aware URL)
    /// Example: "http::unix///tmp/app.sock/" or "http::pipe//myapp/"
    pub url: String,

    /// Window width
    #[serde(default = "default_width")]
    pub width: u32,

    /// Window height
    #[serde(default = "default_height")]
    pub height: u32,

    /// Window title (defaults to app name)
    pub title: Option<String>,

    /// Whether the window is resizable
    #[serde(default = "default_resizable")]
    pub resizable: bool,

    /// Whether to show the window frame
    #[serde(default = "default_decorated")]
    pub decorated: bool,

    /// Whether to start fullscreen
    #[serde(default)]
    pub fullscreen: bool,

    /// Minimum window size
    pub min_size: Option<(u32, u32)>,

    /// Maximum window size
    pub max_size: Option<(u32, u32)>,
}

fn default_width() -> u32 {
    1024
}

fn default_height() -> u32 {
    768
}

fn default_resizable() -> bool {
    true
}

fn default_decorated() -> bool {
    true
}

/// Additional settings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SettingsConfig {
    /// Enable developer tools
    #[serde(default)]
    pub devtools: bool,

    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Custom user agent string
    pub user_agent: Option<String>,
}

fn default_log_level() -> String {
    "info".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config() {
        let toml = r#"
            [app]
            name = "Test App"
            version = "1.0.0"

            [backend]
            command = "gunicorn"
            args = ["--bind", "unix:/tmp/test.sock", "app:app"]
            socket = "/tmp/test.sock"

            [frontend]
            url = "http::unix///tmp/test.sock/"
            width = 1200
            height = 800
        "#;

        let config = HarborConfig::from_str(toml).unwrap();
        assert_eq!(config.app.name, "Test App");
        assert_eq!(config.backend.command, "gunicorn");
        assert_eq!(config.frontend.width, 1200);
    }

    #[test]
    fn test_defaults() {
        let toml = r#"
            [app]
            name = "Minimal App"

            [backend]
            command = "python"
            args = ["server.py"]
            socket = "/tmp/minimal.sock"

            [frontend]
            url = "http::unix///tmp/minimal.sock/"
        "#;

        let config = HarborConfig::from_str(toml).unwrap();
        assert_eq!(config.app.version, "0.1.0");
        assert_eq!(config.frontend.width, 1024);
        assert_eq!(config.frontend.height, 768);
        assert!(config.frontend.resizable);
    }
}
