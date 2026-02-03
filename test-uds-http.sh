#!/bin/bash
# Test Flask HTTP over UDS

SOCKET_PATH="/tmp/hello-harbor.sock"

# Cleanup
pkill -f "gunicorn.*hello-harbor" 2>/dev/null || true
rm -f "$SOCKET_PATH"
sleep 1

# Start backend
cd examples/hello-flask
gunicorn --bind "unix:$SOCKET_PATH" app:app &
BACKEND_PID=$!
cd ../..

echo "Backend PID: $BACKEND_PID"
sleep 2

if [ ! -S "$SOCKET_PATH" ]; then
    echo "Socket not created!"
    exit 1
fi

echo ""
echo "Testing HTTP request over UDS with Host: localhost..."
echo ""

printf "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n" | nc -U "$SOCKET_PATH" | head -30

echo ""
echo ""
echo "Cleanup..."
kill $BACKEND_PID
rm -f "$SOCKET_PATH"
