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
export RUST_LOG=info
cargo run --features servo -- --example hello-flask
