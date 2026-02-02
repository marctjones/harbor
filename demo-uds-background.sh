#!/bin/bash
# Harbor UDS Demo - Background process approach (more reliable)

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

SOCKET_PATH="/tmp/hello-harbor.sock"
FLASK_DIR="examples/hello-flask"
BACKEND_PID=""

cleanup() {
    echo -e "\n${YELLOW}Cleaning up...${NC}"

    if [ -n "$BACKEND_PID" ]; then
        echo "Stopping Flask backend (PID $BACKEND_PID)..."
        kill $BACKEND_PID 2>/dev/null || true
    fi

    pkill -f "gunicorn.*$SOCKET_PATH" 2>/dev/null || true
    rm -f "$SOCKET_PATH"

    echo -e "${GREEN}Cleanup complete${NC}"
}

trap cleanup EXIT INT TERM

echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║        Harbor Unix Domain Socket Demo                 ║${NC}"
echo -e "${BLUE}║  Flask Web App over UDS (No TCP networking!)          ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check dependencies
echo -e "${BLUE}[1/6]${NC} Checking dependencies..."
if ! command -v gunicorn &> /dev/null; then
    echo -e "${RED}Error: gunicorn not found. Install with: pip install gunicorn${NC}"
    exit 1
fi
echo -e "${GREEN}✓ All dependencies present${NC}"
echo ""

# Clean up
echo -e "${BLUE}[2/6]${NC} Preparing Unix socket..."
pkill -f "gunicorn.*$SOCKET_PATH" 2>/dev/null || true
sleep 1
rm -f "$SOCKET_PATH"
echo -e "${GREEN}✓ Socket path clear: $SOCKET_PATH${NC}"
echo ""

# Start Flask backend in background
echo -e "${BLUE}[3/6]${NC} Starting Flask backend on Unix socket..."
cd "$FLASK_DIR"

# Start gunicorn in background and capture PID
gunicorn --bind "unix:$SOCKET_PATH" app:app &
BACKEND_PID=$!
cd ../..

echo "Backend PID: $BACKEND_PID"
echo "Waiting for socket to be ready..."

# Wait for socket to exist
for i in {1..20}; do
    if [ -S "$SOCKET_PATH" ]; then
        echo -e "${GREEN}✓ Socket created!${NC}"
        break
    fi
    sleep 0.5
done

if [ ! -S "$SOCKET_PATH" ]; then
    echo -e "${RED}Error: Socket not created${NC}"
    exit 1
fi

# Give it a moment to be fully ready
sleep 1
echo ""

# Show socket info
echo -e "${BLUE}[4/6]${NC} Socket information:"
ls -lh "$SOCKET_PATH"
echo ""

# Test with curl
echo -e "${BLUE}[5/6]${NC} Testing connection with curl..."
if curl --unix-socket "$SOCKET_PATH" http://localhost/ -I 2>&1 | head -1; then
    echo -e "${GREEN}✓ Flask app responding!${NC}"
else
    echo -e "${RED}Error: Flask app not responding${NC}"
    exit 1
fi
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

# Launch Harbor (blocks until window closes)
cargo run --features servo --quiet -- --example hello-flask

echo ""
echo -e "${GREEN}Demo complete!${NC}"
