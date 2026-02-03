#!/bin/bash
# Simple test to see what Harbor is doing

SOCKET_PATH="/tmp/hello-harbor.sock"

echo "=== Harbor Test ==="
echo ""

# Cleanup
pkill -f "gunicorn.*hello-harbor" 2>/dev/null || true
rm -f "$SOCKET_PATH"
sleep 1

# Start backend
echo "Starting Flask backend on $SOCKET_PATH..."
cd examples/hello-flask
gunicorn --bind "unix:$SOCKET_PATH" app:app &
BACKEND_PID=$!
cd ../..
echo "Backend PID: $BACKEND_PID"

# Wait for socket
sleep 2
if [ -S "$SOCKET_PATH" ]; then
    echo "✓ Socket created"
    ls -l "$SOCKET_PATH"
else
    echo "✗ Socket not created!"
    exit 1
fi

# Test backend
echo ""
echo "Testing backend with curl..."
curl --unix-socket "$SOCKET_PATH" http://localhost/ -I

echo ""
echo "Starting Harbor..."
echo "Look for [UnixConnector] messages in the output"
echo ""

# Run Harbor (let all output show)
cargo run --features servo -- --example hello-flask

# Cleanup
kill $BACKEND_PID 2>/dev/null || true
rm -f "$SOCKET_PATH"
