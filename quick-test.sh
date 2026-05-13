#!/bin/bash
# Quick test with debug output

SOCKET_PATH="/tmp/hello-harbor.sock"

# Cleanup
pkill -f "gunicorn.*hello-harbor" 2>/dev/null || true
pkill -f "target/debug/harbor" 2>/dev/null || true
rm -f "$SOCKET_PATH"
sleep 1

echo "=== Starting Flask Backend ==="
cd examples/hello-flask
gunicorn --bind "unix:$SOCKET_PATH" app:app &
BACKEND_PID=$!
cd ../..
sleep 2

echo "Backend PID: $BACKEND_PID"
echo ""
echo "=== Launching Harbor (watch for [UnixConnection] messages) ==="
echo ""

# Run Harbor - this will show all the debug output
timeout 10 cargo run --features servo --quiet -- --example hello-flask 2>&1 | grep -E "\[Unix|Initial URL|Parser\]" || true

echo ""
echo ""
echo "=== Cleanup ==="
kill $BACKEND_PID 2>/dev/null || true
pkill -f "gunicorn.*hello-harbor" 2>/dev/null || true
pkill -f "target/debug/harbor" 2>/dev/null || true
rm -f "$SOCKET_PATH"
