#!/bin/bash
# Debug Harbor UDS Demo - Shows detailed logging

set -e

SOCKET_PATH="/tmp/hello-harbor.sock"

echo "=== Harbor Debug Demo ==="
echo ""

# Cleanup
echo "[1/4] Cleaning up..."
pkill -f "gunicorn.*hello-harbor" 2>/dev/null || true
rm -f "$SOCKET_PATH"
sleep 1

# Start Flask backend
echo "[2/4] Starting Flask backend..."
cd examples/hello-flask
gunicorn --bind "unix:$SOCKET_PATH" app:app &
BACKEND_PID=$!
cd ../..

echo "       Backend PID: $BACKEND_PID"
echo "       Waiting for socket..."
for i in {1..20}; do
    if [ -S "$SOCKET_PATH" ]; then
        echo "       ✓ Socket created!"
        break
    fi
    sleep 0.5
done

# Test with curl
echo ""
echo "[3/4] Testing backend..."
echo "       $ curl --unix-socket $SOCKET_PATH http://localhost/"
if curl --unix-socket "$SOCKET_PATH" http://localhost/ 2>&1 | head -3; then
    echo "       ✓ Backend responding!"
else
    echo "       ✗ Backend not responding!"
    kill $BACKEND_PID
    exit 1
fi

# Launch Harbor with debug logging
echo ""
echo "[4/4] Launching Harbor with debug logging..."
echo "       Check for [UnixConnector] messages in output"
echo ""

RUST_LOG=info cargo run --features servo -- --example hello-flask 2>&1 | grep -E "(UnixConnector|URL|Initializing|Starting)"

# Cleanup
echo ""
echo "Cleaning up..."
kill $BACKEND_PID 2>/dev/null || true
pkill -f "gunicorn.*hello-harbor" 2>/dev/null || true
rm -f "$SOCKET_PATH"
echo "Done!"
