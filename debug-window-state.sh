#!/bin/bash
# Debug window state while Harbor is running

echo "============================================"
echo "  Window State Debugger"
echo "============================================"
echo ""
echo "This script will:"
echo "1. Start Harbor in the background"
echo "2. Check GNOME window manager state"
echo "3. Report findings"
echo ""

# Start Harbor in background
echo "Starting Harbor..."
RUST_LOG=rigging=debug,harbor=debug cargo run --features servo -- --example hello-flask > /tmp/harbor-debug.log 2>&1 &
HARBOR_PID=$!

echo "Harbor PID: $HARBOR_PID"
echo "Waiting 5 seconds for window creation..."
sleep 5

echo ""
echo "=== CHECKING WINDOW STATE ==="
echo ""

# Check if process is still running
if ps -p $HARBOR_PID > /dev/null; then
    echo "✓ Harbor process is running (PID $HARBOR_PID)"
else
    echo "✗ Harbor process died!"
    exit 1
fi

# Check X11 windows (via XWayland)
echo ""
echo "--- X11/XWayland Windows ---"
if command -v xdotool &> /dev/null; then
    echo "Windows with 'Rigging' in name:"
    xdotool search --name "Rigging" 2>/dev/null || echo "  None found"

    echo ""
    echo "Windows with 'org.rigging' in class:"
    xdotool search --class "org.rigging" 2>/dev/null || echo "  None found"
else
    echo "xdotool not installed"
fi

# Check wmctrl
if command -v wmctrl &> /dev/null; then
    echo ""
    echo "All windows (wmctrl):"
    wmctrl -l | grep -i rigging || echo "  No Rigging windows"
else
    echo "wmctrl not installed"
fi

# Check GNOME Shell windows
echo ""
echo "--- GNOME Shell Windows ---"
gdbus call --session --dest org.gnome.Shell \
    --object-path /org/gnome/Shell \
    --method org.gnome.Shell.Eval \
    "global.get_window_actors().map(a => a.meta_window.get_wm_class()).join(', ')" 2>&1 | head -5

# Check logs for window creation
echo ""
echo "--- Harbor Log (last 50 lines) ---"
tail -50 /tmp/harbor-debug.log | grep -E "(Window|visible|focus|maximized|minimized|WINDOW|EVENT)" || echo "No window-related logs yet"

echo ""
echo "============================================"
echo ""
echo "Full log available at: /tmp/harbor-debug.log"
echo ""
echo "To see live log: tail -f /tmp/harbor-debug.log"
echo "To kill Harbor: kill $HARBOR_PID"
echo ""
echo "Try these:"
echo "  - Check your taskbar for 'Rigging' icon"
echo "  - Press Super key (Activities) and look for window"
echo "  - Press Alt+Tab to see if window appears"
echo "  - Click the taskbar icon if you see it"
echo ""
read -p "Press Enter to stop Harbor and exit..."

kill $HARBOR_PID 2>/dev/null
echo "Harbor stopped."
