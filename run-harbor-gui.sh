#!/bin/bash
# Harbor GUI Demo
# Run this from any terminal to see the GUI window

cd /home/marc/Projects/harbor

echo "============================================"
echo "  🚢 Harbor GUI Demo - Hello Flask  🚢"
echo "============================================"
echo ""
echo "Starting Harbor with GUI window..."
echo ""
echo "You should see:"
echo "  ✓ A window titled 'Hello Harbor!'"
echo "  ✓ Dark blue gradient background"
echo "  ✓ Interactive Flask web app"
echo "  ✓ Content served over Unix socket"
echo ""
echo "Press Ctrl+C to stop"
echo "============================================"
echo ""

# Run Harbor with Flask example
# RUST_LOG levels: trace, debug, info, warn, error
# Set to debug for detailed flow, trace for very verbose
export RUST_LOG=rigging=debug,harbor=debug,servo=info,webrender=info
cargo run --features servo -- --example hello-flask
