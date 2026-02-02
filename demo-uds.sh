#!/bin/bash
# Harbor Unix Socket Demo
# Demonstrates Harbor connecting to Flask app over Unix Domain Socket

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

SOCKET_PATH="/tmp/hello-harbor.sock"
FLASK_DIR="examples/hello-flask"
BACKEND_PID=""

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}Cleaning up...${NC}"

    # Kill backend if we started it
    if [ -n "$BACKEND_PID" ]; then
        echo "Stopping Flask backend (PID $BACKEND_PID)..."
        kill $BACKEND_PID 2>/dev/null || true
    fi

    # Remove socket file
    if [ -S "$SOCKET_PATH" ]; then
        echo "Removing socket file..."
        rm -f "$SOCKET_PATH"
    fi

    echo -e "${GREEN}Cleanup complete${NC}"
}

# Set up trap for cleanup on exit
trap cleanup EXIT INT TERM

echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║        Harbor Unix Domain Socket Demo                 ║${NC}"
echo -e "${BLUE}║  Flask Web App over UDS (No TCP networking!)          ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if we're in the right directory
if [ ! -d "$FLASK_DIR" ]; then
    echo -e "${RED}Error: Must run from Harbor root directory${NC}"
    echo "Usage: ./demo-uds.sh"
    exit 1
fi

# Check dependencies
echo -e "${BLUE}[1/6]${NC} Checking dependencies..."
if ! command -v gunicorn &> /dev/null; then
    echo -e "${RED}Error: gunicorn not found${NC}"
    echo "Install with: pip install gunicorn"
    exit 1
fi

if ! command -v curl &> /dev/null; then
    echo -e "${RED}Error: curl not found${NC}"
    echo "Install with: sudo apt install curl"
    exit 1
fi

if [ ! -f "target/debug/harbor" ]; then
    echo -e "${YELLOW}Warning: harbor binary not found, building...${NC}"
    cargo build --features servo
fi

echo -e "${GREEN}✓ All dependencies present${NC}"
echo ""

# Clean up any existing socket
echo -e "${BLUE}[2/6]${NC} Preparing Unix socket..."
if [ -S "$SOCKET_PATH" ]; then
    echo "Removing existing socket..."
    rm -f "$SOCKET_PATH"
fi
echo -e "${GREEN}✓ Socket path clear: $SOCKET_PATH${NC}"
echo ""

# Start Flask backend
echo -e "${BLUE}[3/6]${NC} Starting Flask backend on Unix socket..."
cd "$FLASK_DIR"
gunicorn --bind "unix:$SOCKET_PATH" --daemon --pid /tmp/harbor-demo.pid app:app
BACKEND_PID=$(cat /tmp/harbor-demo.pid)
cd ../..

echo "Backend PID: $BACKEND_PID"
echo "Waiting for backend to be ready..."
sleep 2

# Check if socket was created
if [ ! -S "$SOCKET_PATH" ]; then
    echo -e "${RED}Error: Socket not created${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Flask backend running on unix://$SOCKET_PATH${NC}"
echo ""

# Test with curl
echo -e "${BLUE}[4/6]${NC} Testing connection with curl..."
echo "Command: curl --unix-socket $SOCKET_PATH http://localhost/ -I"
if curl --unix-socket "$SOCKET_PATH" http://localhost/ -I 2>&1 | head -1; then
    echo -e "${GREEN}✓ Flask app responding!${NC}"
else
    echo -e "${RED}Error: Flask app not responding${NC}"
    exit 1
fi
echo ""

# Show socket info
echo -e "${BLUE}[5/6]${NC} Socket information:"
echo "Path:        $SOCKET_PATH"
echo "Permissions: $(ls -l $SOCKET_PATH | awk '{print $1}')"
echo "Owner:       $(ls -l $SOCKET_PATH | awk '{print $3":"$4}')"
echo ""

# Launch Harbor
echo -e "${BLUE}[6/6]${NC} Launching Harbor..."
echo ""
echo -e "${GREEN}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║  Harbor will now display the Flask app                ║${NC}"
echo -e "${GREEN}║  URL: http::unix://$SOCKET_PATH/     ║${NC}"
echo -e "${GREEN}║                                                        ║${NC}"
echo -e "${GREEN}║  Close the window to exit and clean up                ║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════╝${NC}"
echo ""

# Launch Harbor (this blocks until window closes)
cargo run --features servo -- --example hello-flask

echo ""
echo -e "${GREEN}Demo complete!${NC}"
