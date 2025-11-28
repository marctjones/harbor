# Harbor ⚓

Local desktop app framework - web frontends connecting to gunicorn/nginx over Unix sockets (Linux/macOS) or named pipes (Windows).

## Overview

Harbor enables building desktop applications where:
- **Frontend**: Servo-powered web view
- **Backend**: Any HTTP server (gunicorn, nginx, Flask, FastAPI, etc.)
- **Transport**: Unix Domain Sockets (Linux/macOS) or Named Pipes (Windows)

This architecture provides:
- **Security**: No network exposure - backend only accessible via local IPC
- **Performance**: Lower latency than TCP loopback
- **Simplicity**: Use familiar web technologies

## Quick Start

### 1. Create Configuration

```toml
# app.toml
[app]
name = "My App"
version = "1.0.0"

[backend]
command = "gunicorn"
args = ["--bind", "unix:/tmp/myapp.sock", "-w", "2", "app:create_app()"]
socket = "/tmp/myapp.sock"

[frontend]
url = "http::unix///tmp/myapp.sock/"
width = 1200
height = 800
```

### 2. Create Backend (Flask example)

```python
# app.py
from flask import Flask

def create_app():
    app = Flask(__name__)

    @app.route('/')
    def index():
        return '''
        <html>
        <head><title>My Harbor App</title></head>
        <body>
            <h1>Welcome to Harbor!</h1>
            <p>Running on Unix socket</p>
        </body>
        </html>
        '''

    return app

if __name__ == '__main__':
    app = create_app()
    app.run()
```

### 3. Run

```bash
harbor app.toml
```

## Configuration Reference

### `[app]` Section

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Application name |
| `version` | string | No | Version (default: "0.1.0") |
| `icon` | path | No | Application icon |
| `description` | string | No | Description |

### `[backend]` Section

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `command` | string | Yes | Command to start backend |
| `args` | array | No | Command arguments |
| `socket` | string | Yes | Socket path (Unix) or pipe name (Windows) |
| `workdir` | path | No | Working directory |
| `env` | table | No | Environment variables |
| `startup_timeout` | int | No | Seconds to wait (default: 30) |
| `restart_on_crash` | bool | No | Auto-restart (default: true) |

### `[frontend]` Section

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `url` | string | Yes | Transport-aware URL |
| `width` | int | No | Window width (default: 1024) |
| `height` | int | No | Window height (default: 768) |
| `title` | string | No | Window title (default: app name) |
| `resizable` | bool | No | Allow resize (default: true) |
| `decorated` | bool | No | Show frame (default: true) |
| `fullscreen` | bool | No | Start fullscreen (default: false) |

### `[settings]` Section

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `devtools` | bool | No | Enable devtools (default: false) |
| `log_level` | string | No | Log level (default: "info") |
| `user_agent` | string | No | Custom user agent |

## URL Format

Harbor uses transport-aware URLs from the Rigging library:

```
http::unix///tmp/app.sock/           # Unix socket (absolute path)
http::unix//var/run/app.sock/        # Unix socket (relative path)
http::pipe//myapp/                   # Windows named pipe
```

## Backend Examples

### Gunicorn (Python)
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
args = ["--socket", "/tmp/app.sock", "--wsgi-file", "app.py"]
socket = "/tmp/app.sock"
```

### Nginx
```toml
[backend]
command = "nginx"
args = ["-c", "/path/to/nginx.conf"]
socket = "/tmp/app.sock"
```

### Node.js
```toml
[backend]
command = "node"
args = ["server.js"]
socket = "/tmp/app.sock"
env = { SOCKET_PATH = "/tmp/app.sock" }
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Harbor Application                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────┐         ┌──────────────────────────┐  │
│  │     Frontend     │   UDS   │        Backend           │  │
│  │  (Servo WebView) │◄───────►│  (gunicorn/nginx/etc)    │  │
│  │                  │         │                          │  │
│  │  - HTML/CSS/JS   │         │  - Flask/FastAPI/etc     │  │
│  │  - User Input    │         │  - Business Logic        │  │
│  │  - Rendering     │         │  - Data Processing       │  │
│  └──────────────────┘         └──────────────────────────┘  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Security

Harbor's architecture provides inherent security:

1. **No Network Exposure**: Backend only listens on Unix socket
2. **File Permissions**: Socket has user-only access (0600)
3. **Process Isolation**: Backend runs as child process
4. **No Ports**: No TCP ports to scan or attack

## Comparison with Electron/Tauri

| Feature | Harbor | Electron | Tauri |
|---------|--------|----------|-------|
| Engine | Servo | Chromium | WebView |
| Backend | Any HTTP server | Node.js | Rust |
| Bundle Size | ~20MB | ~150MB | ~3MB |
| Memory | Lower | Higher | Lower |
| Transport | UDS/Named Pipe | IPC | IPC |
| Frontend | Standard Web | Standard Web | Standard Web |

## Platform Support

- **Linux**: Unix Domain Sockets ✅
- **macOS**: Unix Domain Sockets ✅
- **Windows**: Named Pipes (planned)

## License

Mozilla Public License 2.0 (MPL-2.0)

## Related Projects

- [Compass](https://github.com/marctjones/compass) - Privacy-focused browser
- [Corsair](https://github.com/marctjones/corsair) - Tor daemon
- [Rigging](https://github.com/marctjones/rigging) - Transport library
