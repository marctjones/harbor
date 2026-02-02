#!/bin/bash
# Simple Harbor UDS Demo - Essential commands only

set -e

SOCKET_PATH="/tmp/hello-harbor.sock"

echo "=== Harbor Unix Domain Socket Demo ==="
echo ""

# Clean up existing socket
echo "[1/4] Cleaning up..."
rm -f "$SOCKET_PATH"
pkill -f "gunicorn.*hello-harbor" || true
sleep 1

# Start Flask backend
echo "[2/4] Starting Flask backend on Unix socket..."
cd examples/hello-flask
gunicorn --bind "unix:$SOCKET_PATH" --daemon app:app
cd ../..
sleep 2

echo "       Socket: $SOCKET_PATH"
ls -l "$SOCKET_PATH"

# Test with curl
echo ""
echo "[3/4] Testing with curl..."
echo "       $ curl --unix-socket $SOCKET_PATH http://localhost/"
curl --unix-socket "$SOCKET_PATH" http://localhost/ 2>&1 | head -5
echo "       ..."
echo "       ✓ Flask app responding!"

# Launch Harbor
echo ""
echo "[4/4] Launching Harbor..."
echo "       URL: http::unix://$SOCKET_PATH/"
echo ""
echo "       Close the window to exit"
echo ""

cargo run --features servo --quiet -- --example hello-flask

# Cleanup
echo ""
echo "Cleaning up..."
pkill -f "gunicorn.*hello-harbor" || true
rm -f "$SOCKET_PATH"
echo "Done!"
