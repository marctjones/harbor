# Harbor Unix Domain Socket Demo

This guide shows how to run Harbor with a Flask web application over Unix Domain Sockets.

## Quick Start

The fastest way to see Harbor in action:

```bash
# From Harbor root directory
./demo-uds-simple.sh
```

This will:
1. Start Flask backend on Unix socket
2. Test it with curl
3. Launch Harbor to display it
4. Clean up when you close the window

## Demo Scripts

### Full Demo (Recommended)

```bash
./demo-uds.sh
```

Features:
- Pretty colored output
- Detailed progress information
- Automatic cleanup on exit (Ctrl+C or window close)
- Error checking at each step
- Shows socket permissions and ownership

### Simple Demo

```bash
./demo-uds-simple.sh
```

Features:
- Minimal output
- Essential commands only
- Quick execution

### Manual Demo (Step by Step)

If you want to run each command manually:

```bash
# 1. Clean up any existing socket
rm -f /tmp/hello-harbor.sock
pkill -f "gunicorn.*hello-harbor" || true

# 2. Start Flask backend on Unix socket
cd examples/hello-flask
gunicorn --bind "unix:/tmp/hello-harbor.sock" --daemon app:app
cd ../..

# 3. Verify socket was created
ls -l /tmp/hello-harbor.sock

# 4. Test with curl
curl --unix-socket /tmp/hello-harbor.sock http://localhost/

# 5. Launch Harbor
cargo run --features servo -- --example hello-flask

# 6. Cleanup (after closing window)
pkill -f "gunicorn.*hello-harbor"
rm -f /tmp/hello-harbor.sock
```

## What You'll See

### Terminal Output

```
=== Harbor Unix Domain Socket Demo ===

[1/4] Cleaning up...
[2/4] Starting Flask backend on Unix socket...
       Socket: /tmp/hello-harbor.sock
srwxr-xr-x 1 user user 0 Feb 02 14:35 /tmp/hello-harbor.sock

[3/4] Testing with curl...
       $ curl --unix-socket /tmp/hello-harbor.sock http://localhost/
<!DOCTYPE html>
<html lang="en">
<head>
    <title>Hello Harbor!</title>
...
       ✓ Flask app responding!

[4/4] Launching Harbor...
       URL: http::unix:///tmp/hello-harbor.sock/

       Close the window to exit
```

### Harbor Window

You'll see a window displaying the Flask web application:
- Title: "Hello Harbor!"
- Content: Beautiful gradient background with "⚓ Hello, Harbor!" message
- System information showing the Unix socket path
- Interactive counter button
- About section explaining Harbor

### What Makes This Special

**No TCP networking!** The entire stack uses Unix Domain Sockets:

```
Flask App (gunicorn)
    ↓
Unix Socket (/tmp/hello-harbor.sock)
    ↓
Harbor (Servo browser engine)
    ↓
HTTP over UDS
    ↓
Rendered in window
```

Benefits:
- **No network exposure** - Backend only accessible locally
- **Lower latency** - ~40% faster than TCP loopback
- **Better security** - File system permissions control access
- **Familiar web tech** - Standard HTTP, HTML, CSS, JavaScript

## Verifying the Connection

While Harbor is running, you can verify the connection in another terminal:

```bash
# Check running processes
ps aux | grep harbor
ps aux | grep gunicorn

# Check socket
ls -l /tmp/hello-harbor.sock

# Test socket directly
curl --unix-socket /tmp/hello-harbor.sock http://localhost/

# Check what's connected to the socket
lsof /tmp/hello-harbor.sock
```

## Troubleshooting

### Socket already in use

```bash
# Kill any existing gunicorn processes
pkill -f gunicorn

# Remove socket file
rm -f /tmp/hello-harbor.sock
```

### Harbor won't build

```bash
# Make sure Servo feature is enabled
cargo build --features servo

# Check Rust version (should be 1.70+)
rustc --version
```

### Backend not starting

```bash
# Check if gunicorn is installed
which gunicorn

# Install if missing
pip install gunicorn flask

# Verify Flask app works
cd examples/hello-flask
python app.py
```

### Window doesn't appear

Check GNOME Wayland compatibility:
- Harbor forces X11 backend for compatibility
- Window should appear via XWayland
- Check with: `echo $XDG_SESSION_TYPE` (should show "wayland" or "x11")

## Development

### Building

```bash
# Build with Servo support
cargo build --features servo

# Build release version
cargo build --release --features servo
```

### Running Examples

```bash
# Built-in hello-flask example
cargo run --features servo -- --example hello-flask

# Custom app.toml
cargo run --features servo -- path/to/app.toml

# Backend only (for testing)
cargo run --features servo -- --backend-only --example hello-flask
```

### Debug Logging

```bash
# Verbose logging
cargo run --features servo -- --log-level debug --example hello-flask

# Trace logging (very verbose)
cargo run --features servo -- --log-level trace --example hello-flask
```

## Next Steps

After running the demo:

1. **Create your own app** - Use `harbor init` to create a new app
2. **Modify the Flask example** - Edit `examples/hello-flask/app.py`
3. **Try different backends** - Django, FastAPI, Node.js, etc.
4. **Explore transport URLs** - See `TRANSPORT_URLS.md` for URL format details

## Related Documentation

- [README.md](README.md) - Project overview
- [CLAUDE.md](CLAUDE.md) - Development guide
- [examples/hello-flask/](examples/hello-flask/) - Flask example source code
