#!/bin/bash
# Display environment diagnostics

echo "============================================"
echo "  Display Environment Diagnostics"
echo "============================================"
echo ""

echo "--- Session Type ---"
echo "XDG_SESSION_TYPE: $XDG_SESSION_TYPE"
echo "WAYLAND_DISPLAY: $WAYLAND_DISPLAY"
echo "DISPLAY: $DISPLAY"
echo ""

echo "--- Desktop Environment ---"
echo "XDG_CURRENT_DESKTOP: $XDG_CURRENT_DESKTOP"
echo "DESKTOP_SESSION: $DESKTOP_SESSION"
echo "GDMSESSION: $GDMSESSION"
echo ""

echo "--- Window Manager / Compositor ---"
if command -v wmctrl &> /dev/null; then
    echo "Active window manager (wmctrl -m):"
    wmctrl -m 2>/dev/null || echo "  wmctrl failed"
else
    echo "wmctrl not installed"
fi
echo ""

echo "--- Display Resolution ---"
if command -v xrandr &> /dev/null; then
    echo "Connected displays (xrandr):"
    xrandr --current 2>/dev/null | grep " connected" || echo "  xrandr failed (expected on Wayland)"
else
    echo "xrandr not installed"
fi
echo ""

if command -v wlr-randr &> /dev/null; then
    echo "Wayland displays (wlr-randr):"
    wlr-randr 2>/dev/null || echo "  wlr-randr failed"
else
    echo "wlr-randr not installed"
fi
echo ""

echo "--- Window List ---"
if command -v wmctrl &> /dev/null; then
    echo "Open windows (wmctrl -l):"
    wmctrl -l 2>/dev/null || echo "  wmctrl failed (expected on Wayland)"
else
    echo "wmctrl not available"
fi
echo ""

echo "--- Graphics Driver ---"
echo "OpenGL Renderer:"
glxinfo -B 2>/dev/null | grep -E "(OpenGL vendor|OpenGL renderer|OpenGL version)" || echo "  glxinfo failed"
echo ""

echo "--- Wayland Compositor Info ---"
if [ "$XDG_SESSION_TYPE" = "wayland" ]; then
    echo "Compositor: Wayland"
    echo "Checking for GNOME/Mutter:"
    ps aux | grep -i mutter | grep -v grep || echo "  Not running Mutter"
    echo "Checking for KDE/KWin:"
    ps aux | grep -i kwin | grep -v grep || echo "  Not running KWin"
    echo "Checking for Sway:"
    ps aux | grep -i sway | grep -v grep || echo "  Not running Sway"
fi
echo ""

echo "--- Winit/Wayland Compatibility ---"
echo "Checking Wayland backend support..."
if [ "$XDG_SESSION_TYPE" = "wayland" ]; then
    echo "✓ Running on Wayland"
    if [ -n "$WAYLAND_DISPLAY" ]; then
        echo "✓ WAYLAND_DISPLAY set to: $WAYLAND_DISPLAY"
    else
        echo "✗ WAYLAND_DISPLAY not set (this is unusual)"
    fi
else
    echo "✗ Not running Wayland (XDG_SESSION_TYPE=$XDG_SESSION_TYPE)"
fi
echo ""

echo "--- Running Processes (window-related) ---"
echo "Checking for Harbor/Rigging processes:"
ps aux | grep -E "(harbor|rigging|servo)" | grep -v grep || echo "  None found"
echo ""

echo "============================================"
