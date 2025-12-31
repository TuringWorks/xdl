#!/bin/bash
# Script to publish remaining XDL crates to crates.io
# Run this after the rate limit resets (after 18:36:31 GMT on Dec 31, 2025)
#
# The crates must be published in dependency order.
# Add sleep between publishes to avoid rate limits.

set -e

echo "Publishing remaining XDL crates to crates.io..."
echo "Note: Sleeping 30 seconds between each publish to avoid rate limits"
echo ""

# Tier 1: Depend only on xdl-core
echo "=== Publishing Tier 1 crates (depend on xdl-core) ==="
cargo publish -p xdl-parser && sleep 30
cargo publish -p xdl-runtime && sleep 30
cargo publish -p xdl-ffi && sleep 30
cargo publish -p xdl-viz3d && sleep 30
cargo publish -p xdl-viz3d-threejs && sleep 30
cargo publish -p xdl-database && sleep 30

# Tier 2: Depend on Tier 1
echo "=== Publishing Tier 2 crates ==="
cargo publish -p xdl-dataframe && sleep 30
cargo publish -p xdl-lsp && sleep 30

# Tier 3: xdl-stdlib depends on multiple Tier 1 crates
echo "=== Publishing Tier 3 (xdl-stdlib) ==="
cargo publish -p xdl-stdlib && sleep 30

# Tier 4: xdl-interpreter
echo "=== Publishing Tier 4 (xdl-interpreter) ==="
cargo publish -p xdl-interpreter && sleep 30

# Tier 5: Final binaries
echo "=== Publishing Tier 5 (CLI and GUI) ==="
cargo publish -p xdl-cli && sleep 30
cargo publish -p xdl-gui && sleep 30

# Optional crates (Tauri-based, no internal deps)
echo "=== Publishing optional crates ==="
cargo publish -p xdl-chart-viewer && sleep 30
cargo publish -p xdl-desktop-viewer

echo ""
echo "All crates published successfully!"
echo "Check https://crates.io/crates/xdl-core for your published crates"
