# Harbor Blank Window Troubleshooting

## Current Issue

Harbor window opens but displays blank content despite successful Unix socket connection.

## What We Know

### ✅ Working
- Flask backend starts and listens on Unix socket (`/tmp/hello-harbor.sock`)
- Socket file is created with correct permissions
- UnixConnector successfully establishes connection to the socket
- Servo window opens via X11/XWayland
- No crashes or panics

### ❌ Not Working
- Window displays no content (completely blank/white)
- No HTTP request is sent over the socket (no Write logs)
- No HTTP response is received (no Read logs)
- Window is unresponsive - close button doesn't work
- Must use Ctrl+C to force exit

## Debug Output

From `demo-uds-background.sh`:

```
[Parser] Original URL: http::unix///tmp/hello-harbor.sock/
[Parser] Converted URL: http://localhost/
```

This is correct - transport URL converted for Servo's parser.

```
[UnixConnector] Attempting to connect to URI: http://localhost/
[UnixConnector] No socket path in transport URL, using default: "/tmp/hello-harbor.sock"
[UnixConnector] Connecting to socket: "/tmp/hello-harbor.sock"
[UnixConnector] Successfully connected!
```

Connection succeeds! But then... nothing. No write, no read.

## Hypothesis

**The HTTP request is never being sent.**

Possible causes:

1. **Servo isn't navigating** - Maybe `http://localhost/` is treated specially
2. **Async task not polled** - HTTP request future might not be polled
3. **Request blocked** - HSTS, CSP, or other security blocking localhost
4. **Client not configured** - FlexibleConnector might not be used for this request

## Next Steps to Debug

### 1. Verify HTTP works over UDS manually

```bash
# Start Flask
cd examples/hello-flask
gunicorn --bind unix:/tmp/hello-harbor.sock app:app &
PID=$!

# Test with netcat
printf "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n" | nc -U /tmp/hello-harbor.sock

# Cleanup
kill $PID
```

Expected: Should see HTML response

### 2. Add logging in Servo's HTTP loader

Edit `/home/marc/servo/components/net/http_loader.rs` around line 888:

```rust
pub async fn http_fetch(
    // ... params
) -> Response {
    eprintln!("[HTTP_LOADER] Starting fetch for URL: {}", request.current_url());
    // ... rest of function
}
```

### 3. Check if request reaches the HTTP client

Edit `/home/marc/servo/components/net/connector.rs` in FlexibleConnector::call():

```rust
fn call(&mut self, uri: Uri) -> Self::Future {
    eprintln!("[FlexibleConnector] call() invoked for URI: {}", uri);
    match self {
        // ... rest
    }
}
```

### 4. Verify Servo is trying to navigate

Add logging in the running app state or window to see if navigate() is called.

## Alternative Approaches

### Approach 1: Use actual domain instead of localhost

Instead of converting to `http://localhost/`, try using a fake domain:

```rust
fn convert_transport_url_for_parser(url: &str) -> String {
    if url.contains("::unix") {
        "http://harbor.local/".to_string()  // Instead of localhost
    }
    // ...
}
```

Then ensure the connector handles `harbor.local` requests.

### Approach 2: Don't convert the URL at all

Try keeping the original `http::unix://` URL and patch Servo's URL parser
to accept it. This is more invasive but might work better.

### Approach 3: Use HTTP proxy approach

Instead of direct connector, implement an in-process HTTP proxy that:
1. Listens on localhost:random_port
2. Forwards to Unix socket
3. Servo connects to localhost:port as normal HTTP

## Files to Check

- `/home/marc/servo/components/net/http_loader.rs` - HTTP request logic
- `/home/marc/servo/components/net/fetch/methods.rs` - Fetch implementation
- `/home/marc/servo/components/net/resource_thread.rs` - Resource loading
- `/home/marc/Projects/rigging/src/servoshell/desktop/app.rs` - Navigation setup
- `/home/marc/Projects/rigging/src/servoshell/running_app_state.rs` - Page loading

## Quick Test Commands

```bash
# Test with increased logging
RUST_LOG=debug ./demo-uds-background.sh 2>&1 | grep -E "HTTP|fetch|load|navigate"

# Test with strace to see actual system calls
strace -e connect,sendto,recvfrom -f cargo run --features servo -- --example hello-flask 2>&1 | grep unix

# Check if Servo is even trying to make network requests
strace -e socket,connect,send,recv -f cargo run --features servo -- --example hello-flask
```

## Related Issues

This is similar to issues seen when:
- Servo's HSTS list blocks localhost
- CSP headers prevent navigation
- Resource loading is blocked by security policies
- Async tasks aren't being polled (event loop issue)

## Success Criteria

We'll know it's fixed when we see:

1. `[UnixConnection] Wrote X bytes` - HTTP request sent
2. `[UnixConnection] Request: GET / HTTP/1.1` - Request line visible
3. `[UnixConnection] Read X bytes` - Response received
4. Window displays Flask app content
5. Window responds to close button

