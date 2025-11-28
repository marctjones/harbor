/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Harbor CLI - Run local desktop apps with web frontends
//!
//! Usage:
//!   harbor <app.toml>              Run an app from config file
//!   harbor --example hello-flask   Run a built-in example
//!   harbor --help                  Show help

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use harbor::{BrowserConfig, HarborApp, HarborConfig, run_browser, is_browser_available};
use log::{info, warn};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "harbor")]
#[command(author = "The Servo Project Developers")]
#[command(version)]
#[command(about = "Local desktop app framework - web frontends over Unix sockets")]
struct Cli {
    /// Path to app.toml configuration file
    #[arg(value_name = "CONFIG")]
    config: Option<PathBuf>,

    /// Run a built-in example
    #[arg(long, value_name = "NAME")]
    example: Option<String>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Just start the backend, don't open a window (for testing)
    #[arg(long)]
    backend_only: bool,

    /// Print the URL and exit (for integration with other tools)
    #[arg(long)]
    print_url: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Harbor app in the current directory
    Init {
        /// Application name
        #[arg(short, long)]
        name: Option<String>,
    },
    /// List available examples
    Examples,
    /// Validate a configuration file
    Check {
        /// Path to configuration file
        config: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(&cli.log_level),
    )
    .format_timestamp_millis()
    .init();

    // Handle subcommands
    if let Some(command) = cli.command {
        return match command {
            Commands::Init { name } => init_app(name),
            Commands::Examples => list_examples(),
            Commands::Check { config } => check_config(&config),
        };
    }

    // Determine config source
    let config = if let Some(example_name) = cli.example {
        get_example_config(&example_name)?
    } else if let Some(config_path) = cli.config {
        HarborConfig::load(&config_path)
            .with_context(|| format!("Failed to load config: {}", config_path.display()))?
    } else {
        // Try to find app.toml in current directory
        let default_path = PathBuf::from("app.toml");
        if default_path.exists() {
            HarborConfig::load(&default_path)
                .with_context(|| "Failed to load app.toml")?
        } else {
            eprintln!("Usage: harbor <app.toml>");
            eprintln!("       harbor --example hello-flask");
            eprintln!("       harbor --help");
            std::process::exit(1);
        }
    };

    // Create and run the app
    let mut app = HarborApp::new(config);

    info!("Starting Harbor app: {}", app.name());

    let run_config = app.run().with_context(|| "Failed to start app")?;

    if cli.print_url {
        println!("{}", run_config.url);
        // Keep backend running until interrupted
        wait_for_interrupt();
        return Ok(());
    }

    if cli.backend_only {
        info!("Backend running at socket: {}", app.socket_path());
        info!("URL: {}", run_config.url);
        info!("Press Ctrl+C to stop");
        wait_for_interrupt();
        return Ok(());
    }

    // Check if browser support is available
    if !is_browser_available() {
        warn!("Browser support not available (Servo feature disabled)");
        println!();
        println!("=== Harbor App Ready (Backend Only) ===");
        println!("App:    {}", run_config.title);
        println!("URL:    {}", run_config.url);
        println!("Socket: {}", app.socket_path());
        println!();
        println!("Browser support is not enabled. To test the backend:");
        println!("  curl --unix-socket {} http://localhost/", app.socket_path());
        println!();
        println!("To enable browser support, rebuild with: cargo build --features servo");
        println!();
        println!("Press Ctrl+C to stop the backend.");
        wait_for_interrupt();
        return Ok(());
    }

    // Create browser configuration from run config
    let browser_config = BrowserConfig::new(&run_config.url)
        .with_title(&run_config.title)
        .with_size(run_config.width, run_config.height)
        .with_resizable(run_config.resizable)
        .with_decorated(run_config.decorated)
        .with_fullscreen(run_config.fullscreen)
        .with_devtools(run_config.devtools);

    info!("Launching browser window...");

    // Run the browser (this blocks until the window is closed)
    let browser_result = run_browser(browser_config, Some(Box::new(move |event| {
        info!("Browser event: {:?}", event);
    })));

    match browser_result {
        Ok(()) => {
            info!("Browser closed normally");
        }
        Err(e) => {
            warn!("Browser error: {}", e);
            // Fall back to backend-only mode
            println!();
            println!("=== Browser Error - Running Backend Only ===");
            println!("Error: {}", e);
            println!();
            println!("App:    {}", run_config.title);
            println!("URL:    {}", run_config.url);
            println!("Socket: {}", app.socket_path());
            println!();
            println!("To test the backend:");
            println!("  curl --unix-socket {} http://localhost/", app.socket_path());
            println!();
            println!("Press Ctrl+C to stop the backend.");
            wait_for_interrupt();
        }
    }

    info!("Shutting down...");
    Ok(())
}

fn wait_for_interrupt() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn init_app(name: Option<String>) -> Result<()> {
    let app_name = name.unwrap_or_else(|| {
        std::env::current_dir()
            .ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "my-app".to_string())
    });

    let config = format!(
        r#"# Harbor App Configuration
# See https://github.com/marctjones/harbor for documentation

[app]
name = "{name}"
version = "0.1.0"

[backend]
# Command to start your backend server
command = "python"
args = ["-m", "flask", "run", "--host=unix:/tmp/{name_lower}.sock"]
socket = "/tmp/{name_lower}.sock"
# workdir = "."  # Uncomment to set working directory

# Environment variables for your backend
[backend.env]
FLASK_APP = "app.py"
FLASK_ENV = "development"

[frontend]
# Transport-aware URL (http::unix:// for Unix sockets)
url = "http::unix///tmp/{name_lower}.sock/"
width = 1200
height = 800
# title = "{name}"  # Defaults to app.name

[settings]
devtools = false
log_level = "info"
"#,
        name = app_name,
        name_lower = app_name.to_lowercase().replace(' ', "-")
    );

    let config_path = PathBuf::from("app.toml");
    if config_path.exists() {
        anyhow::bail!("app.toml already exists in current directory");
    }

    std::fs::write(&config_path, config)?;
    println!("Created app.toml for '{}'", app_name);
    println!();
    println!("Next steps:");
    println!("  1. Edit app.toml to configure your backend");
    println!("  2. Create your backend server (e.g., Flask app)");
    println!("  3. Run: harbor app.toml");

    Ok(())
}

fn list_examples() -> Result<()> {
    println!("Available examples:");
    println!();
    println!("  hello-flask    Simple Flask app demonstrating Harbor basics");
    println!();
    println!("Run an example with: harbor --example <name>");
    Ok(())
}

fn check_config(config_path: &PathBuf) -> Result<()> {
    let config = HarborConfig::load(config_path)
        .with_context(|| format!("Failed to load: {}", config_path.display()))?;

    println!("Configuration valid!");
    println!();
    println!("App:     {} v{}", config.app.name, config.app.version);
    println!("Backend: {} {:?}", config.backend.command, config.backend.args);
    println!("Socket:  {}", config.backend.socket);
    println!("URL:     {}", config.frontend.url);
    println!("Window:  {}x{}", config.frontend.width, config.frontend.height);

    Ok(())
}

fn get_example_config(name: &str) -> Result<HarborConfig> {
    match name {
        "hello-flask" => {
            let toml = r#"
[app]
name = "Hello Flask"
version = "1.0.0"
description = "Simple Flask example for Harbor"

[backend]
command = "python"
args = ["-c", """
from flask import Flask
app = Flask(__name__)

@app.route('/')
def index():
    return '''
<!DOCTYPE html>
<html>
<head>
    <title>Hello Harbor!</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            min-height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
        }
        .container {
            text-align: center;
            padding: 2rem;
        }
        h1 { font-size: 3rem; margin-bottom: 0.5rem; }
        p { font-size: 1.2rem; opacity: 0.9; }
        .emoji { font-size: 4rem; }
    </style>
</head>
<body>
    <div class="container">
        <div class="emoji">⚓</div>
        <h1>Hello, Harbor!</h1>
        <p>Your Flask app is running over Unix Domain Socket</p>
        <p><small>Socket: /tmp/hello-harbor.sock</small></p>
    </div>
</body>
</html>
'''

if __name__ == '__main__':
    import os
    sock = '/tmp/hello-harbor.sock'
    if os.path.exists(sock):
        os.remove(sock)
    app.run(host=f'unix://{sock}')
"""]
socket = "/tmp/hello-harbor.sock"

[backend.env]
FLASK_ENV = "development"
PYTHONUNBUFFERED = "1"

[frontend]
url = "http::unix///tmp/hello-harbor.sock/"
width = 800
height = 600
title = "Hello Harbor!"

[settings]
devtools = false
log_level = "info"
"#;
            HarborConfig::from_str(toml).context("Failed to parse hello-flask example config")
        }
        _ => {
            anyhow::bail!("Unknown example: {}. Run 'harbor examples' to see available examples.", name);
        }
    }
}
